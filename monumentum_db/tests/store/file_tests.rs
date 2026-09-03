use monumentum_db::error::DbError;
use monumentum_db::store::file::{
    append_to_file, open_or_create, read_file, sync_file, write_all_atomic,
};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::common::TempPath;

#[test]
fn open_or_create_new_file() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    let mut file = open_or_create(temp.path())?;
    file.write_all(b"hello")?;
    file.sync_all()?;
    drop(file);

    assert!(temp.path().exists());
    let content = fs::read(temp.path())?;
    assert_eq!(content, b"hello");
    Ok(())
}

#[test]
fn open_or_create_existing_file_not_truncated() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    fs::write(temp.path(), b"initial")?;

    let file = open_or_create(temp.path())?;
    drop(file);

    let content = fs::read(temp.path())?;
    assert_eq!(content, b"initial");
    Ok(())
}

#[test]
fn read_file_existing_returns_content() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    fs::write(temp.path(), b"test data")?;

    let content = read_file(temp.path())?;
    assert_eq!(content, b"test data");
    Ok(())
}

#[test]
fn read_file_non_existent_returns_error() {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    let result = read_file(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
fn write_all_atomic_creates_file() -> Result<(), DbError> {
    let dir = TempPath::new_dir("monumentum_fs_test_dir");
    let path = dir.path().join("subdir").join("file.txt");
    let data = b"atomic write";
    write_all_atomic(&path, data)?;

    let content = fs::read(&path)?;
    assert_eq!(content, data);
    Ok(())
}

#[test]
fn write_all_atomic_overwrites_existing_file() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    fs::write(temp.path(), b"old")?;

    let new_data = b"new data";
    write_all_atomic(temp.path(), new_data)?;

    let content = fs::read(temp.path())?;
    assert_eq!(content, new_data);
    Ok(())
}

#[test]
fn write_all_atomic_path_without_parent_returns_error() {
    let result = write_all_atomic(Path::new(""), b"data");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: path has no parent directory"
        );
    }
}

#[test]
fn append_to_file_appends_data() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;

    append_to_file(&mut file, b"first ")?;
    append_to_file(&mut file, b"second")?;

    file.seek(SeekFrom::Start(0))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    assert_eq!(content, "first second");
    Ok(())
}

#[test]
fn sync_file_succeeds() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_fs_test_file");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    sync_file(&file)?;
    Ok(())
}
