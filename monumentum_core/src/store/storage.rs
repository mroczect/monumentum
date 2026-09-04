use crate::catalog::Catalog;
use crate::serde::{Decode, Encode};
use crate::store::file::write_all_atomic;
use crate::store::wal::Wal;
use crate::table::Table;
use monumentum_handler::error::DbError;
use std::io::Cursor;
use std::path::{Path, PathBuf};
pub trait StorageEngine {
    fn get_catalog(&self) -> &Catalog;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
}

#[derive(Debug)]
pub struct FileStorage {
    data_path: PathBuf,
    wal: Wal,
    catalog: Catalog,
    current_seq: u64,
}

impl FileStorage {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let data_path = path.to_path_buf();
        let mut wal_path = path.to_path_buf();
        let _ = wal_path.set_extension("wal");

        let mut wal = Wal::open(&wal_path)?;

        let (mut catalog, mut current_seq) = if data_path.exists() {
            let data = std::fs::read(&data_path)?;
            let (seq, cat) = decode_snapshot(&data)?;
            (cat, seq)
        } else {
            (Catalog::new(), 0)
        };

        let records = wal.read_all()?;
        for record in records {
            let (seq, cat) = decode_snapshot(&record)?;
            if seq > current_seq {
                current_seq = seq;
                catalog = cat;
            }
        }

        Ok(Self {
            data_path,
            wal,
            catalog,
            current_seq,
        })
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.wal.sync()
    }

    pub fn checkpoint(&mut self) -> Result<(), DbError> {
        let data = encode_snapshot(self.current_seq, &self.catalog)?;
        write_all_atomic(&self.data_path, &data)?;
        self.wal.truncate()?;
        Ok(())
    }

    pub fn reload_from_disk(&mut self) -> Result<Catalog, DbError> {
        let data = std::fs::read(&self.data_path)?;
        let (snapshot_seq, snapshot_cat) = decode_snapshot(&data)?;
        let records = self.wal.read_all()?;

        let mut current_seq = snapshot_seq;
        let mut catalog = snapshot_cat;
        for record in records {
            let (record_seq, record_cat) = decode_snapshot(&record)?;
            if record_seq > current_seq {
                current_seq = record_seq;
                catalog = record_cat;
            }
        }

        self.catalog = catalog.clone();
        self.current_seq = current_seq;
        Ok(catalog)
    }

    pub fn close(mut self) -> Result<(), DbError> {
        self.wal.unlock()?;
        Ok(())
    }
}

impl StorageEngine for FileStorage {
    fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        let new_seq = self
            .current_seq
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("sequence number overflow"))?;
        let data = encode_snapshot(new_seq, catalog)?;
        self.wal.append(&data)?;
        self.current_seq = new_seq;
        self.catalog = catalog.clone();
        Ok(())
    }

    fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }
}

#[derive(Debug, Default)]
pub struct InMemoryStorage {
    catalog: Catalog,
}

impl InMemoryStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl StorageEngine for InMemoryStorage {
    fn get_catalog(&self) -> &Catalog {
        &self.catalog
    }

    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        self.catalog = catalog.clone();
        Ok(())
    }

    fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }
}

fn encode_snapshot(seq: u64, catalog: &Catalog) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::with_capacity(64);
    seq.encode(&mut buf)?;
    catalog.encode(&mut buf)?;
    Ok(buf)
}

fn decode_snapshot(data: &[u8]) -> Result<(u64, Catalog), DbError> {
    let mut cursor = Cursor::new(data);
    let seq = u64::decode(&mut cursor)?;
    let catalog = Catalog::decode(&mut cursor)?;
    Ok((seq, catalog))
}
