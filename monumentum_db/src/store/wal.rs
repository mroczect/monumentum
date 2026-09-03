use crate::error::DbError;
use crate::store::append_log::{append_record, read_records};
use crate::store::file::{open_or_create, sync_file};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Wal {
    file: Option<File>,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let file = open_or_create(path)?;

        #[cfg(unix)]
        {
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                return Err(DbError::Io(err));
            }
        }

        Ok(Self { file: Some(file) })
    }

    pub fn append(&mut self, payload: &[u8]) -> Result<(), DbError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| DbError::invalid_operation("WAL is already unlocked"))?;
        append_record(file, payload)?;
        sync_file(file)?;
        Ok(())
    }

    pub fn sync(&self) -> Result<(), DbError> {
        if let Some(file) = &self.file {
            sync_file(file)?;
            Ok(())
        } else {
            Err(DbError::invalid_operation("WAL is already unlocked"))
        }
    }

    pub fn read_all(&mut self) -> Result<Vec<Vec<u8>>, DbError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| DbError::invalid_operation("WAL is already unlocked"))?;
        read_records(file)
    }

    pub fn truncate(&mut self) -> Result<(), DbError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| DbError::invalid_operation("WAL is already unlocked"))?;
        file.set_len(0)?;
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<(), DbError> {
        if let Some(file) = self.file.take() {
            #[cfg(unix)]
            {
                let fd = file.as_raw_fd();
                let ret = unsafe { libc::flock(fd, libc::LOCK_UN) };
                if ret != 0 {
                    let err = std::io::Error::last_os_error();
                    return Err(DbError::Io(err));
                }
            }
        }
        Ok(())
    }
}

impl Drop for Wal {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            #[cfg(unix)]
            {
                let fd = file.as_raw_fd();
                unsafe {
                    libc::flock(fd, libc::LOCK_UN);
                }
            }
        }
    }
}
