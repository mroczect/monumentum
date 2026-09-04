use crate::error::DbError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

#[cfg(target_os = "linux")]
const O_NOFOLLOW: i32 = 0x20000;
#[cfg(not(target_os = "linux"))]
const O_NOFOLLOW: i32 = 0x100;

pub fn open_or_create(path: &Path) -> Result<File, DbError> {
    let mut options = OpenOptions::new();
    let _ = options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        let _ = options.mode(0o600);
        let _ = options.custom_flags(O_NOFOLLOW);
    }
    let file = options.open(path)?;
    Ok(file)
}

pub fn read_file(path: &Path) -> Result<Vec<u8>, DbError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn write_all_atomic(path: &Path, data: &[u8]) -> Result<(), DbError> {
    let parent = path
        .parent()
        .ok_or_else(|| DbError::invalid_operation("path has no parent directory"))?;
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = unique_tmp_path(
        parent,
        path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp"),
    );

    {
        let mut tmp_options = OpenOptions::new();
        let _ = tmp_options.write(true).create_new(true);
        #[cfg(unix)]
        {
            let _ = tmp_options.mode(0o600);
            let _ = tmp_options.custom_flags(O_NOFOLLOW);
        }
        let mut tmp_file = tmp_options.open(&tmp_path)?;
        tmp_file.write_all(data)?;
        tmp_file.sync_all()?;
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(DbError::from_io(e));
    }

    if let Ok(dir) = File::open(parent) {
        dir.sync_all()?;
    }

    Ok(())
}

fn unique_tmp_path(parent: &Path, base_name: &str) -> PathBuf {
    use core::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{}.{}.{}.tmp",
        base_name,
        std::process::id(),
        count
    ))
}

pub fn append_to_file(file: &mut File, data: &[u8]) -> Result<(), DbError> {
    file.write_all(data)?;
    Ok(())
}

pub fn sync_file(file: &File) -> Result<(), DbError> {
    file.sync_all()?;
    Ok(())
}
