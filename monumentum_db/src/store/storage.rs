use crate::core::catalog::Catalog;
use crate::core::table::Table;
use crate::error::DbError;
use crate::store::file::write_all_atomic;
use crate::store::serialize::{decode_catalog, encode_catalog};
use crate::store::wal::Wal;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
}

#[derive(Debug)]
pub struct FileStorage {
    data_path: PathBuf,
    wal: Wal,
    catalog: Catalog,
}

impl FileStorage {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let data_path = path.to_path_buf();
        let mut wal_path = path.to_path_buf();
        wal_path.set_extension("wal");

        let mut wal = Wal::open(&wal_path)?;

        // Muat katalog dari data utama jika ada
        let mut catalog = Catalog::new();
        if data_path.exists() {
            let data = std::fs::read(&data_path)?;
            let mut cursor = Cursor::new(&data[..]);
            catalog = decode_catalog(&mut cursor)?;
        }

        // Replay WAL: ambil record terakhir sebagai state terbaru
        if let Ok(records) = wal.read_all()
            && let Some(last_record) = records.last()
        {
            let mut cursor = Cursor::new(last_record.as_slice());
            catalog = decode_catalog(&mut cursor)?;
        }

        Ok(Self {
            data_path,
            wal,
            catalog,
        })
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.wal.sync()
    }

    pub fn checkpoint(&mut self) -> Result<(), DbError> {
        let data = encode_catalog(&self.catalog);
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
        let data = encode_catalog(catalog);
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
