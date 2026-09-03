use crate::common::TempPath;
use monumentum_db::error::DbError;
use monumentum_db::store::append_log::append_record;
use monumentum_db::store::recovery::recover_wal;
use proptest::prelude::*;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

fn create_wal_file(temp: &TempPath) -> Result<std::fs::File, DbError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())
        .map_err(DbError::from)
}

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
    let mut file = create_wal_file(&temp)?;
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
    let mut file = create_wal_file(&temp)?;
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
    let mut file = create_wal_file(&temp)?;
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

#[test]
fn recover_wal_on_directory_returns_error() {
    let dir = TempPath::new_dir("monumentum_recovery_dir");
    let result = recover_wal(dir.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
fn recover_wal_with_many_records_returns_all_in_order() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_many");
    let mut file = create_wal_file(&temp)?;

    let mut expected = Vec::new();
    for i in 0..1000 {
        let payload = format!("record-{i}").into_bytes();
        append_record(&mut file, &payload)?;
        expected.push(payload);
    }
    drop(file);

    let recovery = recover_wal(temp.path())?;
    assert_eq!(recovery.records.len(), expected.len());
    for (actual, expected) in recovery.records.iter().zip(expected.iter()) {
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn recover_wal_with_empty_payload_record() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_empty_payload");
    let mut file = create_wal_file(&temp)?;
    append_record(&mut file, b"")?;
    drop(file);

    let recovery = recover_wal(temp.path())?;
    assert_eq!(recovery.records.len(), 1);
    assert!(recovery.records[0].is_empty());
    Ok(())
}

#[test]
fn recover_wal_with_record_followed_by_garbage_returns_error() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_partial_corrupt");
    let mut file = create_wal_file(&temp)?;
    append_record(&mut file, b"good")?;

    file.write_all(&[0xFF; 10])?;
    file.sync_all()?;
    drop(file);

    let result = recover_wal(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    Ok(())
}

#[test]
fn recover_wal_with_truncated_payload_returns_error() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_truncated");
    let mut file = create_wal_file(&temp)?;
    let payload = b"this is a payload".to_vec();
    append_record(&mut file, &payload)?;
    drop(file);

    let metadata = std::fs::metadata(temp.path())?;
    let len = metadata.len();
    let f = OpenOptions::new().write(true).open(temp.path())?;
    f.set_len(len - 5)?;
    f.sync_all()?;
    drop(f);

    let result = recover_wal(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    Ok(())
}

#[test]
fn recover_wal_with_corrupt_checksum_returns_error() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_recovery_test_corrupt_checksum");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    let payload = b"data".to_vec();
    append_record(&mut file, &payload)?;
    drop(file);

    let mut corrupt_file = OpenOptions::new().write(true).open(temp.path())?;
    corrupt_file.seek(SeekFrom::Start(16))?;
    corrupt_file.write_all(&[0u8; 4])?;
    corrupt_file.sync_all()?;
    drop(corrupt_file);

    let result = recover_wal(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Data corruption: checksum mismatch in log record"
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    #[test]
    fn recover_wal_roundtrip_multiple_records(
        records in proptest::collection::vec(
            proptest::collection::vec(proptest::prelude::any::<u8>(), 0..1000),
            0..20
        )
    ) {
        let temp = TempPath::new_file("monumentum_recovery_prop");
        let mut file = create_wal_file(&temp).unwrap();
        for record in &records {
            append_record(&mut file, record).unwrap();
        }
        drop(file);

        let recovery = recover_wal(temp.path()).unwrap();
        prop_assert_eq!(recovery.records, records);
    }

    #[test]
    fn recover_wal_single_random_record(
        payload in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..10000)
    ) {
        let temp = TempPath::new_file("monumentum_recovery_prop_single");
        let mut file = create_wal_file(&temp).unwrap();
        append_record(&mut file, &payload).unwrap();
        drop(file);

        let recovery = recover_wal(temp.path()).unwrap();
        prop_assert_eq!(recovery.records.len(), 1);
        prop_assert_eq!(recovery.records[0].as_slice(), payload.as_slice());
    }
}
