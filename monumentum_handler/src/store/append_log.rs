use crate::constants::MAX_RECORD_SIZE;
use crate::error::DbError;
use crate::store::file::{append_to_file, sync_file};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const MAGIC: u32 = 0x4D4F_4E55;
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

pub fn append_record(file: &mut File, payload: &[u8]) -> Result<(), DbError> {
    if payload.len() > MAX_RECORD_SIZE {
        return Err(DbError::invalid_operation(format!(
            "record size {} exceeds maximum allowed {}",
            payload.len(),
            MAX_RECORD_SIZE
        )));
    }

    let length = payload.len() as u64;
    let checksum = crc32(payload);
    let mut header = [0_u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&length.to_le_bytes());
    header[16..20].copy_from_slice(&checksum.to_le_bytes());

    append_to_file(file, &header)?;
    append_to_file(file, payload)?;
    sync_file(file)?;
    Ok(())
}

fn read_exact(file: &mut File, buf: &mut [u8]) -> Result<(), DbError> {
    let mut bytes_read = 0;
    while bytes_read < buf.len() {
        let remaining = buf.get_mut(bytes_read..).ok_or_else(|| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid buffer range",
            ))
        })?;
        match file.read(remaining) {
            Ok(0) => {
                return Err(DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "unexpected end of file",
                )));
            }
            Ok(n) => bytes_read = bytes_read.saturating_add(n),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(DbError::from_io(e)),
        }
    }
    Ok(())
}

fn read_payload(file: &mut File, length: usize) -> Result<Vec<u8>, DbError> {
    let mut payload = vec![0_u8; length];
    read_exact(file, &mut payload)?;
    Ok(payload)
}

pub fn read_records(file: &mut File) -> Result<Vec<Vec<u8>>, DbError> {
    let mut records = Vec::new();
    let _ = file.seek(SeekFrom::Start(0))?;

    loop {
        let mut header_buf = [0_u8; HEADER_SIZE];
        let bytes_read = file.read(&mut header_buf[..])?;
        if bytes_read == 0 {
            return Ok(records);
        }
        if bytes_read != HEADER_SIZE {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected end of file in log header",
            )));
        }

        let magic = u32::from_le_bytes(header_buf[0..4].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid header slice: {e}"),
            ))
        })?);
        if magic != MAGIC {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid magic in log header",
            )));
        }

        let version = u32::from_le_bytes(header_buf[4..8].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid header slice: {e}"),
            ))
        })?);
        if version != VERSION {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported log version {version}"),
            )));
        }

        let length_u64 = u64::from_le_bytes(header_buf[8..16].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid header slice: {e}"),
            ))
        })?);
        let length = usize::try_from(length_u64).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("record length too large for platform: {e}"),
            ))
        })?;
        if length > MAX_RECORD_SIZE {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "record length exceeds maximum allowed",
            )));
        }

        let expected_checksum = u32::from_le_bytes(header_buf[16..20].try_into().map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid header slice: {e}"),
            ))
        })?);

        let payload = read_payload(file, length)?;

        let actual_checksum = crc32(&payload);
        if actual_checksum != expected_checksum {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "checksum mismatch in log record",
            )));
        }

        records.push(payload);
    }
}
