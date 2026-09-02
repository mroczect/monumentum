use crate::error::DbError;
use crate::store::file::{append_to_file, sync_file};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const MAGIC: u32 = 0x4D4F4E55;
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20;
const MAX_RECORD_SIZE: usize = 64 * 1024 * 1024;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
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
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&length.to_le_bytes());
    header[16..20].copy_from_slice(&checksum.to_le_bytes());

    append_to_file(file, &header)?;
    append_to_file(file, payload)?;
    sync_file(file)?;
    Ok(())
}

pub fn read_records(file: &mut File) -> Result<Vec<Vec<u8>>, DbError> {
    let mut records = Vec::new();
    let mut header_buf = [0u8; HEADER_SIZE];
    file.seek(SeekFrom::Start(0))?;

    loop {
        let mut bytes_read = 0;
        while bytes_read < HEADER_SIZE {
            match file.read(&mut header_buf[bytes_read..]) {
                Ok(0) => {
                    if bytes_read == 0 {
                        return Ok(records);
                    } else {
                        return Err(DbError::corruption(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "unexpected end of file in log header",
                        )));
                    }
                }
                Ok(n) => bytes_read += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(DbError::Io(e)),
            }
        }

        let magic = u32::from_le_bytes(
            header_buf
                .get(0..4)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "header too short",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid header slice",
                    ))
                })?,
        );
        if magic != MAGIC {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid magic in log header",
            )));
        }
        let version = u32::from_le_bytes(
            header_buf
                .get(4..8)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "header too short",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid header slice",
                    ))
                })?,
        );
        if version != VERSION {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported log version {version}"),
            )));
        }
        let length = u64::from_le_bytes(
            header_buf
                .get(8..16)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "header too short",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid header slice",
                    ))
                })?,
        ) as usize;
        if length > MAX_RECORD_SIZE {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "record length exceeds maximum allowed",
            )));
        }
        let expected_checksum = u32::from_le_bytes(
            header_buf
                .get(16..20)
                .ok_or_else(|| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "header too short",
                    ))
                })?
                .try_into()
                .map_err(|_| {
                    DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid header slice",
                    ))
                })?,
        );

        let mut payload = vec![0u8; length];
        let mut bytes_read = 0;
        while bytes_read < length {
            match file.read(&mut payload[bytes_read..]) {
                Ok(0) => {
                    return Err(DbError::corruption(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "unexpected end of file in log payload",
                    )));
                }
                Ok(n) => bytes_read += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(DbError::Io(e)),
            }
        }

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
