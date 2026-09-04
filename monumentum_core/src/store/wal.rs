use crate::store::append_log::{append_record, read_records};
use crate::store::file::{open_or_create, sync_file};
use fs2::FileExt;
use monumentum_handler::error::DbError;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub struct Wal {
    file: Option<File>,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let file = open_or_create(path)?;
        file.try_lock_exclusive().map_err(DbError::from_io)?;
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
        let _ = file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<(), DbError> {
        if let Some(file) = self.file.take() {
            file.unlock().map_err(DbError::from_io)?;
        }
        Ok(())
    }
}

impl Drop for Wal {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            let _ = file.unlock();
        }
    }
}
