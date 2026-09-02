#![cfg_attr(test, allow(unused_crate_dependencies))]
use monumentum_db::core::catalog::Catalog;
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
