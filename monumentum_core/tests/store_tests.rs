#![allow(unused_crate_dependencies)]

use monumentum_core::store::append_log::{WalRecordType, append_record, read_records};
use monumentum_core::store::file::{open_or_create, read_file, write_all_atomic};
use monumentum_core::store::wal::Wal;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_store_{}_{}.{}",
        std::process::id(),
        nanos,
        ext
    ))
}

#[test]
fn test_append_record_and_read() {
    let path = temp_path("log");
    let mut file = open_or_create(&path).unwrap_or_else(|_| unreachable!());
    let payload = vec![1, 2, 3, 4];
    let append_result = append_record(&mut file, &payload);
    assert!(append_result.is_ok());

    let records = read_records(&mut file);
    assert!(records.is_ok());
    if let Ok(records) = records {
        assert_eq!(records.len(), 1);
        if let Some(record) = records.first() {
            assert_eq!(record, &payload);
        } else {
            unreachable!();
        }
    }
    drop(file);
    let _ = fs::remove_file(&path);
}

#[test]
fn test_wal_append_and_read() {
    let path = temp_path("wal");
    let mut wal = Wal::open(&path).unwrap_or_else(|_| unreachable!());
    let append = wal.append_wal_record(1, WalRecordType::PageWrite, &[0xAA, 0xBB]);
    assert!(append.is_ok());

    let records = wal.read_wal_records();
    assert!(records.is_ok());
    if let Ok(records) = records {
        assert_eq!(records.len(), 1);
        if let Some(record) = records.first() {
            assert_eq!(record.lsn, 1);
            assert_eq!(record.record_type, WalRecordType::PageWrite);
            assert_eq!(record.data, vec![0xAA, 0xBB]);
        } else {
            unreachable!();
        }
    }

    let truncate = wal.truncate();
    assert!(truncate.is_ok());
    let records_after = wal.read_wal_records();
    assert!(records_after.is_ok());
    if let Ok(records) = records_after {
        assert_eq!(records.len(), 0);
    }
    drop(wal);
    let _ = fs::remove_file(&path);
}

#[test]
fn test_atomic_write_and_read() {
    let path = temp_path("txt");
    let data = b"hello world".to_vec();
    let write = write_all_atomic(&path, &data);
    assert!(write.is_ok());

    let read = read_file(&path);
    assert!(read.is_ok());
    if let Ok(read_data) = read {
        assert_eq!(read_data, data);
    }
    let _ = fs::remove_file(&path);
}
