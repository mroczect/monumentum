use monumentum_db::error::DbError;
use monumentum_db::store::append_log::{append_record, read_records};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: u32 = 0x4D4F4E55;
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 20;
const MAX_RECORD_SIZE: usize = 64 * 1024 * 1024;

fn temp_file_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let unique = format!("monumentum_log_test_{}_{}", std::process::id(), nanos);
    std::env::temp_dir().join(unique)
}

fn create_test_file() -> Result<(File, PathBuf), DbError> {
    let path = temp_file_path();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    Ok((file, path))
}

fn write_bytes_to_file(file: &mut File, data: &[u8]) -> Result<(), DbError> {
    file.write_all(data)?;
    file.sync_all()?;
    file.seek(SeekFrom::Start(0))?;
    Ok(())
}

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

#[test]
fn crc32_empty_input_returns_zero() {
    assert_eq!(crc32(b""), 0);
}

#[test]
fn crc32_known_value() {
    assert_eq!(
        crc32(b"The quick brown fox jumps over the lazy dog"),
        0x414FA339
    );
}

#[test]
fn crc32_different_data_produce_different_crc() {
    let crc1 = crc32(b"hello");
    let crc2 = crc32(b"world");
    assert_ne!(crc1, crc2);
}

#[test]
fn append_record_success_and_file_content() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload = b"test payload".to_vec();
    append_record(&mut file, &payload)?;

    file.seek(SeekFrom::Start(0))?;
    let mut all_bytes = Vec::new();
    file.read_to_end(&mut all_bytes)?;
    assert_eq!(all_bytes.len(), HEADER_SIZE + payload.len());

    let magic = u32::from_le_bytes(
        all_bytes
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
    assert_eq!(magic, MAGIC);

    let version = u32::from_le_bytes(
        all_bytes
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
    assert_eq!(version, VERSION);

    let length = u64::from_le_bytes(
        all_bytes
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
    );
    assert_eq!(length as usize, payload.len());

    let checksum = u32::from_le_bytes(
        all_bytes
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
    assert_eq!(checksum, crc32(&payload));

    assert_eq!(
        all_bytes
            .get(HEADER_SIZE..)
            .ok_or_else(|| DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "payload missing"
            )))?,
        payload.as_slice()
    );

    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn append_record_payload_too_large_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload = vec![0u8; MAX_RECORD_SIZE + 1];
    let result = append_record(&mut file, &payload);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            format!(
                "Invalid operation: record size {} exceeds maximum allowed {}",
                MAX_RECORD_SIZE + 1,
                MAX_RECORD_SIZE
            )
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn append_multiple_records_and_read_back() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload1 = b"first".to_vec();
    let payload2 = b"second".to_vec();
    append_record(&mut file, &payload1)?;
    append_record(&mut file, &payload2)?;

    let records = read_records(&mut file)?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], payload1);
    assert_eq!(records[1], payload2);
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_empty_file_returns_empty_vec() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let records = read_records(&mut file)?;
    assert!(records.is_empty());
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_truncated_header_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let partial_header = [1u8; 10];
    write_bytes_to_file(&mut file, &partial_header)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: unexpected end of file in log header"
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_invalid_magic_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&0u64.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    write_bytes_to_file(&mut file, &header)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: invalid magic in log header"
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_unsupported_version_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&2u32.to_le_bytes());
    header[8..16].copy_from_slice(&0u64.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    write_bytes_to_file(&mut file, &header)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Data corruption: unsupported log version 2");
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_length_too_large_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    let large_len = (MAX_RECORD_SIZE + 1) as u64;
    header[8..16].copy_from_slice(&large_len.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    write_bytes_to_file(&mut file, &header)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: record length exceeds maximum allowed"
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_checksum_mismatch_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload = b"data".to_vec();
    let checksum = crc32(&payload);
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    header[16..20].copy_from_slice(&(checksum.wrapping_add(1)).to_le_bytes());
    let mut data = Vec::new();
    data.extend_from_slice(&header);
    data.extend_from_slice(&payload);
    write_bytes_to_file(&mut file, &data)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: checksum mismatch in log record"
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn read_records_truncated_payload_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload = b"full payload".to_vec();
    let checksum = crc32(&payload);
    let mut header = [0u8; HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    header[16..20].copy_from_slice(&checksum.to_le_bytes());
    let mut data = Vec::new();
    data.extend_from_slice(&header);
    data.extend_from_slice(&payload[..payload.len() / 2]);
    write_bytes_to_file(&mut file, &data)?;

    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: unexpected end of file in log payload"
        );
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}
