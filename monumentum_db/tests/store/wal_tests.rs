use monumentum_db::error::DbError;
use monumentum_db::store::wal::Wal;
use std::fs;

use crate::common::TempPath;


#[test]
fn open_creates_new_file() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;
    let records = wal.read_all()?;
    assert!(records.is_empty());
    wal.unlock()?;

    assert!(temp.path().exists());
    Ok(())
}

#[test]
fn open_existing_file_not_truncated() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    fs::write(temp.path(), b"existing")?;

    let mut wal = Wal::open(temp.path())?;
    let read_result = wal.read_all();
    assert!(read_result.is_err());
    wal.unlock()?;

    let content = fs::read(temp.path())?;
    assert_eq!(content, b"existing");
    Ok(())
}

#[test]
fn append_and_read_back() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    let payload1 = b"first".to_vec();
    let payload2 = b"second".to_vec();

    wal.append(&payload1)?;
    wal.append(&payload2)?;

    let records = wal.read_all()?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0], payload1);
    assert_eq!(records[1], payload2);

    wal.unlock()?;
    Ok(())
}

#[test]
fn append_empty_payload() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    let payload = Vec::new();
    wal.append(&payload)?;

    let records = wal.read_all()?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], payload);

    wal.unlock()?;
    Ok(())
}

#[test]
fn append_multiple_records_and_read_back_in_order() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

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
    Ok(())
}

#[test]
fn sync_succeeds() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    wal.sync()?;
    wal.unlock()?;
    Ok(())
}

#[test]
fn truncate_clears_all_data() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    wal.append(b"data")?;
    wal.truncate()?;

    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    Ok(())
}

#[test]
fn unlock_succeeds() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    wal.unlock()?;
    Ok(())
}

#[test]
fn read_all_on_empty_file_returns_empty() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    Ok(())
}

#[test]
fn read_all_after_truncate_returns_empty() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let mut wal = Wal::open(temp.path())?;

    wal.append(b"something")?;
    wal.truncate()?;
    let records = wal.read_all()?;
    assert!(records.is_empty());

    wal.unlock()?;
    Ok(())
}

#[test]
fn lock_prevents_second_open_until_first_dropped() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_wal_test");
    let wal1 = Wal::open(temp.path())?;

    let path_clone = temp.path().to_path_buf();
    let handle = std::thread::spawn(move || {
        let result = Wal::open(&path_clone);
        result.is_ok()
    });

    std::thread::sleep(std::time::Duration::from_millis(100));
    drop(wal1);

    let second_succeeded = handle.join().expect("thread panicked");
    assert!(second_succeeded, "second open should succeed after lock release");
    Ok(())
}