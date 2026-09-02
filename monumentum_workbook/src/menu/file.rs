use crate::Workbook;
use crate::WorkbookError;
use monumentum_db::core::catalog::Catalog;
use monumentum_db::store::storage::{FileStorage, InMemoryStorage, StorageEngine};
use monumentum_query::formula::FunctionRegistry;
use std::path::Path;

const FILE_EXTENSION: &str = "monumentum";

fn validate_extension(path: &Path) -> Result<(), WorkbookError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case(FILE_EXTENSION) => Ok(()),
        _ => Err(WorkbookError::InvalidExtension),
    }
}

impl Workbook<FileStorage> {
    pub fn open(path: &Path) -> Result<Self, WorkbookError> {
        validate_extension(path)?;
        let mut storage = FileStorage::open(path)?;
        let catalog = storage.load_catalog()?;
        Ok(Self {
            catalog,
            storage,
            functions: FunctionRegistry::new(),
        })
    }

    pub fn create_new(path: &Path) -> Result<Self, WorkbookError> {
        validate_extension(path)?;
        if path.exists() {
            return Err(WorkbookError::FileExists);
        }
        let mut storage = FileStorage::open(path)?;
        let catalog = Catalog::new();
        storage.save_catalog(&catalog)?;
        Ok(Self {
            catalog,
            storage,
            functions: FunctionRegistry::new(),
        })
    }

    pub fn save(&mut self) -> Result<(), WorkbookError> {
        self.storage.save_catalog(&self.catalog)?;
        self.storage.checkpoint()?;
        Ok(())
    }

    pub fn save_as(&mut self, path: &Path) -> Result<(), WorkbookError> {
        validate_extension(path)?;
        if path.exists() {
            return Err(WorkbookError::FileExists);
        }
        let mut new_storage = FileStorage::open(path)?;
        new_storage.save_catalog(&self.catalog)?;
        new_storage.checkpoint()?;
        new_storage.close()?;
        Ok(())
    }

    pub fn save_a_copy(&self, path: &Path) -> Result<(), WorkbookError> {
        validate_extension(path)?;
        if path.exists() {
            return Err(WorkbookError::FileExists);
        }
        let mut new_storage = FileStorage::open(path)?;
        new_storage.save_catalog(&self.catalog)?;
        new_storage.checkpoint()?;
        new_storage.close()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), WorkbookError> {
        self.catalog = self.storage.load_catalog()?;
        Ok(())
    }

    pub fn close(self) -> Result<(), WorkbookError> {
        self.storage.close()?;
        Ok(())
    }
}

impl Workbook<InMemoryStorage> {
    #[must_use]
    pub fn new_in_memory() -> Self {
        let storage = InMemoryStorage::new();
        let catalog = Catalog::new();
        Self {
            catalog,
            storage,
            functions: FunctionRegistry::new(),
        }
    }

    #[must_use]
    pub fn load_in_memory(catalog: Catalog) -> Self {
        let storage = InMemoryStorage::new();
        Self {
            catalog,
            storage,
            functions: FunctionRegistry::new(),
        }
    }

    pub fn save(&mut self) -> Result<(), WorkbookError> {
        self.storage.save_catalog(&self.catalog)?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), WorkbookError> {
        self.catalog = self.storage.load_catalog()?;
        Ok(())
    }

    pub fn close(self) -> Result<(), WorkbookError> {
        Ok(())
    }
}
