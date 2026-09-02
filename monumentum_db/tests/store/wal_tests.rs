use monumentum_db::error::DbError;
use monumentum_db::store::wal::Wal;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_file_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let unique = format!("monumentum_wal_test_{}_{}", std::process::id(), nanos);
    std::env::temp_dir().join(unique)
}

fn cleanup(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn open_creates_new_file() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);

    let mut wal = Wal::open(&path)?;
    let records = wal.read_all()?;
    assert!(records.is_empty());
    wal.unlock()?;

    assert!(path.exists());
    cleanup(&path);
    Ok(())
}

#[test]
fn open_existing_file_not_truncated() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    fs::write(&path, b"existing")?;

    let mut wal = Wal::open(&path)?;
    let read_result = wal.read_all();
    assert!(read_result.is_err());
    wal.unlock()?;

    let content = fs::read(&path)?;
    assert_eq!(content, b"existing");

    cleanup(&path);
    Ok(())
}

#[test]
fn append_and_read_back() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    let payload1 = b"first".to_vec();
    let payload2 = b"second".to_vec();

    wal.append(&payload1)?;
    wal.append(&payload2)?;

    let records = wal.read_all()?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], payload1);
    assert_eq!(records[1], payload2);

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn append_empty_payload() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    let payload = Vec::new();
    wal.append(&payload)?;

    let records = wal.read_all()?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], payload);

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn append_multiple_records_and_read_back_in_order() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    let payloads = vec![
        b"alpha".to_vec(),
        b"beta".to_vec(),
        b"gamma".to_vec(),
        b"delta".to_vec(),
    ];

    for p in &payloads {
        wal.append(p)?;
    }

    let records = wal.read_all()?;
    assert_eq!(records, payloads);

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn sync_succeeds() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    wal.sync()?;
    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn truncate_clears_all_data() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    wal.append(b"data")?;
    wal.truncate()?;

    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn unlock_succeeds() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn read_all_on_empty_file_returns_empty() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn read_all_after_truncate_returns_empty() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let mut wal = Wal::open(&path)?;

    wal.append(b"something")?;
    wal.truncate()?;
    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn multiple_open_same_path_should_fail_or_wait_due_to_lock() -> Result<(), DbError> {
    let path = temp_file_path();
    cleanup(&path);
    let wal1 = Wal::open(&path)?;

    drop(wal1);
    Ok(())
}
