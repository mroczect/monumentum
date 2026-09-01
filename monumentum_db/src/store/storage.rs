use crate::core::catalog::Catalog;
use crate::core::table::Table;
use crate::error::DbError;
use crate::store::serialize::{decode_catalog, encode_catalog};
use crate::store::wal::Wal;
use std::io::Cursor;
use std::path::Path;

pub trait StorageEngine {
    fn load_catalog(&mut self) -> Result<Catalog, DbError>;
    fn save_catalog(&mut self, catalog: &Catalog) -> Result<(), DbError>;
    fn get_table(&self, name: &str) -> Option<&Table>;
    fn get_table_mut(&mut self, name: &str) -> Option<&mut Table>;
}

#[derive(Debug)]
pub struct FileStorage {
    wal: Wal,
    catalog: Catalog,
}

impl FileStorage {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let mut wal = Wal::open(path)?;
        let mut catalog = Catalog::new();
        if let Ok(records) = wal.read_all()
            && let Some(last_record) = records.last()
        {
            let mut cursor = Cursor::new(last_record.as_slice());
            catalog = decode_catalog(&mut cursor)?;
        }
        Ok(Self { wal, catalog })
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.wal.sync()
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
