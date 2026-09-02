use monumentum_db::error::DbError;
use monumentum_db::store::append_log::append_record;
use monumentum_db::store::recovery::recover_wal;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_file_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let unique = format!("monumentum_recovery_test_{}_{}", std::process::id(), nanos);
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

#[test]
fn recover_wal_on_non_existent_path_creates_empty_wal_and_returns_empty() -> Result<(), DbError> {
    let path = temp_file_path();
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }

    let recovery = recover_wal(&path)?;
    assert!(recovery.records.is_empty());
    assert!(path.exists());
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn recover_wal_on_empty_file_returns_empty_records() -> Result<(), DbError> {
    let (_file, path) = create_test_file()?;
    let recovery = recover_wal(&path)?;
    assert!(recovery.records.is_empty());
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn recover_wal_with_single_record_returns_that_record() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload = b"test record".to_vec();
    append_record(&mut file, &payload)?;
    drop(file);

    let recovery = recover_wal(&path)?;
    assert_eq!(recovery.records.len(), 1);
    assert_eq!(recovery.records[0], payload);
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn recover_wal_with_multiple_records_returns_all_in_order() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let payload1 = b"first".to_vec();
    let payload2 = b"second".to_vec();
    let payload3 = b"third".to_vec();
    append_record(&mut file, &payload1)?;
    append_record(&mut file, &payload2)?;
    append_record(&mut file, &payload3)?;
    drop(file);

    let recovery = recover_wal(&path)?;
    assert_eq!(recovery.records.len(), 3);
    assert_eq!(recovery.records[0], payload1);
    assert_eq!(recovery.records[1], payload2);
    assert_eq!(recovery.records[2], payload3);
    std::fs::remove_file(&path).ok();
    Ok(())
}

#[test]
fn recover_wal_on_corrupt_file_returns_error() -> Result<(), DbError> {
    let (mut file, path) = create_test_file()?;
    let garbage = vec![0xFF; 32];
    file.write_all(&garbage)?;
    file.sync_all()?;
    drop(file);

    let result = recover_wal(&path);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    std::fs::remove_file(&path).ok();
    Ok(())
}
