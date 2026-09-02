use monumentum_db::error::DbError;
use monumentum_db::store::file::{
    append_to_file, open_or_create, read_file, sync_file, write_all_atomic,
};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir_path() -> Result<PathBuf, DbError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let unique = format!("monumentum_fs_test_{}_{}", std::process::id(), nanos);
    let dir = std::env::temp_dir().join(unique);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn open_or_create_new_file() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("new_file.txt");
    let mut file = open_or_create(&path)?;
    file.write_all(b"hello")?;
    file.sync_all()?;
    drop(file);

    assert!(path.exists());
    let content = fs::read(&path)?;
    assert_eq!(content, b"hello");
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn open_or_create_existing_file_not_truncated() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("existing.txt");
    fs::write(&path, b"initial")?;

    let file = open_or_create(&path)?;
    drop(file);

    let content = fs::read(&path)?;
    assert_eq!(content, b"initial");
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn read_file_existing_returns_content() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("data.txt");
    fs::write(&path, b"test data")?;

    let content = read_file(&path)?;
    assert_eq!(content, b"test data");
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn read_file_non_existent_returns_error() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("missing.txt");
    let result = read_file(&path);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::Io(_)));
    }
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn write_all_atomic_creates_file() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("subdir").join("file.txt");
    let data = b"atomic write";
    write_all_atomic(&path, data)?;

    let content = fs::read(&path)?;
    assert_eq!(content, data);
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn write_all_atomic_overwrites_existing_file() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("file.txt");
    fs::write(&path, b"old")?;

    let new_data = b"new data";
    write_all_atomic(&path, new_data)?;

    let content = fs::read(&path)?;
    assert_eq!(content, new_data);
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn write_all_atomic_path_without_parent_returns_error() {
    let path = Path::new("");
    let result = write_all_atomic(path, b"data");
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
    let dir = temp_dir_path()?;
    let path = dir.join("append.txt");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    append_to_file(&mut file, b"first ")?;
    append_to_file(&mut file, b"second")?;

    file.seek(SeekFrom::Start(0))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    assert_eq!(content, "first second");
    fs::remove_dir_all(&dir).ok();
    Ok(())
}

#[test]
fn sync_file_succeeds() -> Result<(), DbError> {
    let dir = temp_dir_path()?;
    let path = dir.join("sync.txt");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    sync_file(&file)?;
    fs::remove_dir_all(&dir).ok();
    Ok(())
}
