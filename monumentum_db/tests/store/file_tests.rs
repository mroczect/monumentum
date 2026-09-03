use monumentum_db::error::DbError;
use monumentum_db::store::file::{
    append_to_file, open_or_create, read_file, sync_file, write_all_atomic,
};
use proptest::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::common::TempPath;

fn create_temp_file() -> TempPath {
    TempPath::new_file("monumentum_fs_test_file")
}

fn create_temp_dir() -> TempPath {
    TempPath::new_dir("monumentum_fs_test_dir")
}

#[test]
fn open_or_create_new_file() -> Result<(), DbError> {
    let temp = create_temp_file();
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
    let temp = create_temp_file();
    fs::write(temp.path(), b"initial")?;

    let file = open_or_create(temp.path())?;
    drop(file);

    let content = fs::read(temp.path())?;
    assert_eq!(content, b"initial");
    Ok(())
}

#[test]
fn open_or_create_on_directory_returns_error() {
    let dir = create_temp_dir();
    let result = open_or_create(dir.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
#[cfg(unix)]
fn open_or_create_sets_0600_permissions() -> Result<(), DbError> {
    let temp = create_temp_file();
    let file = open_or_create(temp.path())?;
    drop(file);

    let metadata = fs::metadata(temp.path())?;
    let mode = metadata.permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    Ok(())
}

#[test]
#[cfg(unix)]
fn open_or_create_does_not_follow_symlink() -> Result<(), DbError> {
    let target = create_temp_file();
    fs::write(target.path(), b"target")?;

    let symlink_path = target.path().with_extension("symlink");
    std::os::unix::fs::symlink(target.path(), &symlink_path)?;

    let result = open_or_create(&symlink_path);
    assert!(result.is_err());
    let _ = fs::remove_file(&symlink_path);
    Ok(())
}

#[test]
fn open_or_create_parent_missing_returns_error() {
    let dir = create_temp_dir();
    let nonexistent_subdir = dir.path().join("missing").join("file.txt");
    let result = open_or_create(&nonexistent_subdir);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
fn read_file_existing_returns_content() -> Result<(), DbError> {
    let temp = create_temp_file();
    fs::write(temp.path(), b"test data")?;
    let content = read_file(temp.path())?;
    assert_eq!(content, b"test data");
    Ok(())
}

#[test]
fn read_file_empty_file_returns_empty_vec() -> Result<(), DbError> {
    let temp = create_temp_file();
    fs::write(temp.path(), b"")?;
    let content = read_file(temp.path())?;
    assert!(content.is_empty());
    Ok(())
}

#[test]
fn read_file_non_existent_returns_error() {
    let temp = create_temp_file();
    let result = read_file(temp.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
fn read_file_on_directory_returns_error() {
    let dir = create_temp_dir();
    let result = read_file(dir.path());
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
}

#[test]
fn write_all_atomic_creates_file() -> Result<(), DbError> {
    let dir = create_temp_dir();
    let path = dir.path().join("subdir").join("file.txt");
    let data = b"atomic write";
    write_all_atomic(&path, data)?;

    let content = fs::read(&path)?;
    assert_eq!(content, data);
    Ok(())
}

#[test]
fn write_all_atomic_overwrites_existing_file() -> Result<(), DbError> {
    let temp = create_temp_file();
    fs::write(temp.path(), b"old")?;

    let new_data = b"new data";
    write_all_atomic(temp.path(), new_data)?;

    let content = fs::read(temp.path())?;
    assert_eq!(content, new_data);
    Ok(())
}

#[test]
fn write_all_atomic_with_empty_data() -> Result<(), DbError> {
    let temp = create_temp_file();
    write_all_atomic(temp.path(), b"")?;
    let content = fs::read(temp.path())?;
    assert!(content.is_empty());
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
fn write_all_atomic_parent_is_file_returns_error() -> Result<(), DbError> {
    let parent_file = create_temp_file();
    fs::write(parent_file.path(), b"file")?;
    let child = parent_file.path().join("child.txt");
    let result = write_all_atomic(&child, b"data");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
    Ok(())
}

#[test]
fn write_all_atomic_leaves_no_temp_files() -> Result<(), DbError> {
    let dir = create_temp_dir();
    let target = dir.path().join("target.txt");
    write_all_atomic(&target, b"data")?;

    let entries: Vec<_> = fs::read_dir(dir.path())?.filter_map(Result::ok).collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name(), "target.txt");
    Ok(())
}

#[test]
fn write_all_atomic_failure_removes_temp_file() -> Result<(), DbError> {
    let dir = create_temp_dir();
    let target_dir = dir.path().join("target_dir");
    fs::create_dir(&target_dir)?;

    let result = write_all_atomic(&target_dir, b"data");
    assert!(result.is_err());

    let entries: Vec<_> = fs::read_dir(dir.path())?.filter_map(Result::ok).collect();
    assert_eq!(entries.len(), 1);
    Ok(())
}

#[test]
fn append_to_file_appends_data() -> Result<(), DbError> {
    let temp = create_temp_file();
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
fn append_to_file_on_read_only_file_returns_error() -> Result<(), DbError> {
    let temp = create_temp_file();
    fs::write(temp.path(), b"data")?;
    let mut file = OpenOptions::new().read(true).open(temp.path())?;

    let result = append_to_file(&mut file, b"more");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
    Ok(())
}

#[test]
fn sync_file_succeeds() -> Result<(), DbError> {
    let temp = create_temp_file();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp.path())?;
    sync_file(&file)?;
    Ok(())
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    #[test]
    fn write_all_atomic_roundtrip(
        data in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..10000)
    ) {
        let temp = TempPath::new_file("monumentum_fs_test_prop_write");
        write_all_atomic(temp.path(), &data).unwrap();
        let read_back = read_file(temp.path()).unwrap();
        prop_assert_eq!(data, read_back);
    }

    #[test]
    fn append_to_file_matches_expected(
        chunks in proptest::collection::vec(
            proptest::collection::vec(proptest::prelude::any::<u8>(), 0..1000),
            0..20
        )
    ) {
        let temp = TempPath::new_file("monumentum_fs_test_prop_append");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(temp.path())
            .unwrap();

        let mut expected = Vec::new();
        for chunk in &chunks {
            append_to_file(&mut file, chunk).unwrap();
            expected.extend_from_slice(chunk);
        }

        file.seek(SeekFrom::Start(0)).unwrap();
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        prop_assert_eq!(content, expected);
    }

    #[test]
    fn open_or_create_and_read_file_roundtrip(
        data in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..10000)
    ) {
        let temp = TempPath::new_file("monumentum_fs_test_prop_open");
        let mut file = open_or_create(temp.path()).unwrap();
        file.write_all(&data).unwrap();
        file.sync_all().unwrap();
        drop(file);

        let read_back = read_file(temp.path()).unwrap();
        prop_assert_eq!(data, read_back);
    }
}
