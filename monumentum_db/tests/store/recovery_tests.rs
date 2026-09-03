use monumentum_db::error::DbError;
use monumentum_db::store::append_log::append_record;
use monumentum_db::store::recovery::recover_wal;
use std::fs::OpenOptions;
use std::io::Write;

use crate::common::TempPath;

#[test]
fn recover_wal_on_non_existent_path_creates_empty_wal_and_returns_empty() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test");
    if temp.path().exists() {
        std::fs::remove_file(temp.path())?;
    }

    let recovery = recover_wal(temp.path())?;
    assert!(recovery.records.is_empty());
    assert!(temp.path().exists());
    Ok(())
}

#[test]
fn recover_wal_on_empty_file_returns_empty_records() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    let recovery = recover_wal(temp.path())?;
    assert!(recovery.records.is_empty());
    Ok(())
}

#[test]
fn recover_wal_with_single_record_returns_that_record() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    let payload = b"test record".to_vec();
    append_record(&mut file, &payload)?;
    drop(file);

    let recovery = recover_wal(temp.path())?;
    assert_eq!(recovery.records.len(), 1);
    assert_eq!(recovery.records[0], payload);
    Ok(())
}

#[test]
fn recover_wal_with_multiple_records_returns_all_in_order() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    let payload1 = b"first".to_vec();
    let payload2 = b"second".to_vec();
    let payload3 = b"third".to_vec();
    append_record(&mut file, &payload1)?;
    append_record(&mut file, &payload2)?;
    append_record(&mut file, &payload3)?;
    drop(file);

    let recovery = recover_wal(temp.path())?;
    assert_eq!(recovery.records.len(), 3);
    assert_eq!(recovery.records[0], payload1);
    assert_eq!(recovery.records[1], payload2);
    assert_eq!(recovery.records[2], payload3);
    Ok(())
}

#[test]
fn recover_wal_on_corrupt_file_returns_error() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    let garbage = vec![0xFF; 32];
    file.write_all(&garbage)?;
    file.sync_all()?;
    drop(file);

    let result = recover_wal(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    Ok(())
}
