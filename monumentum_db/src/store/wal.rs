use crate::error::DbError;
use crate::store::append_log::{append_record, read_records};
use crate::store::file::{open_or_create, sync_file};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub struct Wal {
    file: File,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let file = open_or_create(path)?;
        // Kunci file (blocking) untuk mencegah akses bersamaan
        #[cfg(unix)]
        {
            file.lock()?;
        }
        Ok(Self { file })
    }

    pub fn append(&mut self, payload: &[u8]) -> Result<(), DbError> {
        append_record(&mut self.file, payload)?;
        sync_file(&self.file)?;
        Ok(())
    }

    pub fn sync(&self) -> Result<(), DbError> {
        sync_file(&self.file)
    }

    pub fn read_all(&mut self) -> Result<Vec<Vec<u8>>, DbError> {
        read_records(&mut self.file)
    }

    pub fn truncate(&mut self) -> Result<(), DbError> {
        self.file.set_len(0)?;
        self.file.sync_all()?;
        self.file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<(), DbError> {
        #[cfg(unix)]
        {
            self.file.unlock()?;
        }
        Ok(())
    }
}
