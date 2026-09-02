#![cfg_attr(test, allow(unused_crate_dependencies))]
use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::table::Table;
use monumentum_db::store::storage::StorageEngine;
mod error;
pub mod menu;

pub use error::WorkbookError;

#[derive(Debug)]
pub struct Workbook<S: StorageEngine> {
    pub(crate) catalog: Catalog,
    pub(crate) storage: S,
}

impl<S: StorageEngine> Workbook<S> {
    #[must_use]
    pub const fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub const fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }
}

impl<S: StorageEngine> Workbook<S> {
    pub fn sheet(&self, name: &str) -> Result<&Table, WorkbookError> {
        self.catalog.get_table(name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(name).to_string())
        })
    }

    pub fn sheet_mut(&mut self, name: &str) -> Result<&mut Table, WorkbookError> {
        self.catalog.get_table_mut(name).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(name).to_string())
        })
    }
}
