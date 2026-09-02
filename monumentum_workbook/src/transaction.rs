use crate::{Workbook, WorkbookError};
use monumentum_db::core::catalog::Catalog;
use monumentum_db::store::storage::StorageEngine;

#[derive(Debug)]
pub struct Transaction<'a, S: StorageEngine> {
    workbook: &'a mut Workbook<S>,
    snapshot: Catalog,
}

impl<'a, S: StorageEngine> Transaction<'a, S> {
    #[must_use]
    pub fn begin(workbook: &'a mut Workbook<S>) -> Self {
        let snapshot = workbook.catalog().clone();
        Self { workbook, snapshot }
    }

    pub const fn workbook_mut(&mut self) -> &mut Workbook<S> {
        self.workbook
    }

    pub fn commit(self) -> Result<(), WorkbookError> {
        self.workbook.persist_catalog()
    }

    pub fn rollback(self) {
        *self.workbook.catalog_mut() = self.snapshot;
    }
}
