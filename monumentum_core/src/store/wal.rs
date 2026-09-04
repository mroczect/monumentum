use crate::store::append_log::{
    WalRecord, WalRecordType, append_record, append_wal_record, read_records, read_wal_records,
};
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

    pub fn append_wal_record(
        &mut self,
        lsn: u64,
        record_type: WalRecordType,
        data: &[u8],
    ) -> Result<(), DbError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| DbError::invalid_operation("WAL is already unlocked"))?;
        append_wal_record(file, lsn, record_type, data)?;
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

    pub fn read_wal_records(&mut self) -> Result<Vec<WalRecord>, DbError> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| DbError::invalid_operation("WAL is already unlocked"))?;
        read_wal_records(file)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_wal_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!(
            "monumentum_wal_test_{}_{}.wal",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn test_wal_delta_records() -> Result<(), DbError> {
        let path = temp_wal_path();
        let mut wal = Wal::open(&path)?;

        wal.append_wal_record(1, WalRecordType::PageWrite, &[0xAA, 0xBB])?;
        wal.append_wal_record(2, WalRecordType::Snapshot, &[0xCC])?;

        let records = wal.read_wal_records()?;
        assert_eq!(records.len(), 2);
        let first = records
            .first()
            .ok_or_else(|| DbError::invalid_operation("missing record 0"))?;
        let second = records
            .get(1)
            .ok_or_else(|| DbError::invalid_operation("missing record 1"))?;

        assert_eq!(first.lsn, 1);
        assert_eq!(first.record_type, WalRecordType::PageWrite);
        assert_eq!(first.data, vec![0xAA, 0xBB]);
        assert_eq!(second.lsn, 2);
        assert_eq!(second.record_type, WalRecordType::Snapshot);
        assert_eq!(second.data, vec![0xCC]);

        drop(wal);
        let _ = fs::remove_file(&path);
        Ok(())
    }
}
