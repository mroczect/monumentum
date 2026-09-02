use crate::core::catalog::Catalog;
use crate::core::table::Table;
use crate::error::DbError;
use crate::store::file::write_all_atomic;
use crate::store::serialize::{decode_catalog, encode_catalog};
use crate::store::wal::Wal;
use std::path::{Path, PathBuf};

pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
}

const SEQ_BYTES: usize = 8;

fn encode_snapshot(seq: u64, catalog: &Catalog) -> Vec<u8> {
    let mut buf = Vec::with_capacity(SEQ_BYTES + 64);
    buf.extend_from_slice(&seq.to_le_bytes());
    buf.extend_from_slice(&encode_catalog(catalog));
    buf
}

fn decode_snapshot(data: &[u8]) -> Result<(u64, Catalog), DbError> {
    if data.len() < SEQ_BYTES {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "snapshot too short",
        )));
    }
    let seq = u64::from_le_bytes(data[0..SEQ_BYTES].try_into().map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid sequence bytes",
        ))
    })?);
    let catalog_data = &data[SEQ_BYTES..];
    let catalog = decode_catalog(catalog_data)?;
    Ok((seq, catalog))
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
        wal_path.set_extension("wal");

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
            match decode_snapshot(&record) {
                Ok((seq, cat)) if seq > current_seq => {
                    current_seq = seq;
                    catalog = cat;
                }
                _ => {}
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
        let data = encode_snapshot(self.current_seq, &self.catalog);
        write_all_atomic(&self.data_path, &data)?;
        self.wal.truncate()?;
        Ok(())
    }

    pub fn close(mut self) -> Result<(), DbError> {
        self.wal.unlock()?;
        Ok(())
    }
}

impl StorageEngine for FileStorage {
    fn load_catalog(&mut self) -> Result<Catalog, DbError> {
        Ok(self.catalog.clone())
    }

    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        self.current_seq = self.current_seq.wrapping_add(1);
        let data = encode_snapshot(self.current_seq, catalog);
        self.wal.append(&data)?;
        self.catalog = catalog.clone();
        Ok(())
    }

    fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }

    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.catalog.get_table_mut(name)
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
    fn load_catalog(&mut self) -> Result<Catalog, DbError> {
        Ok(self.catalog.clone())
    }

    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError> {
        self.catalog = catalog.clone();
        Ok(())
    }

    fn get_table(&self, name: &str) -> Option<&Table> {
        self.catalog.get_table(name)
    }

    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.catalog.get_table_mut(name)
    }
}
