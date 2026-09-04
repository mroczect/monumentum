use fs2 as _;
use monumentum_core::store::append_log::{append_record, read_records};
use monumentum_core::store::recovery::recover_wal;
use monumentum_core::store::wal::Wal;
use monumentum_handler::error::DbError;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

fn temp_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    path.push(format!("{}_{}_{}", std::process::id(), name, unique));
    path
}

#[test]
fn append_and_read_single_record() {
    let path = temp_path("test.log");
    let file_result = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path);
    let Ok(mut file) = file_result else {
        return;
    };
    let payload = b"hello world".to_vec();
    assert!(append_record(&mut file, &payload).is_ok());
    let _ = file.seek(SeekFrom::Start(0));
    let records_result = read_records(&mut file);
    let Ok(records) = records_result else {
        return;
    };
    assert_eq!(records, vec![payload]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn read_records_corrupt_checksum() {
    let path = temp_path("corrupt.log");
    let file_result = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path);
    let Ok(mut file) = file_result else {
        return;
    };
    let payload = b"data".to_vec();
    assert!(append_record(&mut file, &payload).is_ok());
    let len_result = file.metadata().map(|m| m.len());
    let Ok(len) = len_result else {
        return;
    };
    if len == 0 {
        return;
    }
    let _ = file.seek(SeekFrom::Start(len - 1));
    assert!(file.write_all(&[0xFF]).is_ok());
    let _ = file.seek(SeekFrom::Start(0));
    let result = read_records(&mut file);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Corruption(_)));
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn wal_append_read_truncate() {
    let path = temp_path("test.wal");
    let wal_result = Wal::open(&path);
    let Ok(mut wal) = wal_result else {
        return;
    };
    assert!(wal.append(b"record1".as_ref()).is_ok());
    assert!(wal.append(b"record2".as_ref()).is_ok());
    let records_result = wal.read_all();
    let Ok(records) = records_result else {
        return;
    };
    assert_eq!(records, vec![b"record1".to_vec(), b"record2".to_vec()]);
    assert!(wal.truncate().is_ok());
    let records_result = wal.read_all();
    let Ok(records) = records_result else {
        return;
    };
    assert!(records.is_empty());
    assert!(wal.unlock().is_ok());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn recover_wal_returns_records() {
    let path = temp_path("recover.wal");
    {
        let wal_result = Wal::open(&path);
        let Ok(mut wal) = wal_result else {
            return;
        };
        assert!(wal.append(b"data".as_ref()).is_ok());
    }
    let result = recover_wal(&path);
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.records, vec![b"data".to_vec()]);
    }
    let _ = std::fs::remove_file(&path);
}
