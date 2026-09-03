use crate::{Workbook, WorkbookError};
use core::ops::{Deref, DerefMut};
use monumentum_db::core::catalog::Catalog;
use monumentum_db::store::storage::StorageEngine;

#[derive(Debug)]
pub struct Transaction<'a, S: StorageEngine> {
    workbook: &'a mut Workbook<S>,
    snapshot: Catalog,
    open: bool,
}

impl<'a, S: StorageEngine> Transaction<'a, S> {
    #[must_use]
    pub fn begin(workbook: &'a mut Workbook<S>) -> Self {
        let snapshot = workbook.catalog().clone();
        Self {
            workbook,
            snapshot,
            open: true,
        }
    }

    pub fn commit(mut self) -> Result<(), WorkbookError> {
        self.open = false;
        self.workbook.persist_catalog()
    }

    pub fn rollback(mut self) {
        self.open = false;
        *self.workbook.catalog_mut() = self.snapshot.clone();
    }

    pub const fn workbook_mut(&mut self) -> &mut Workbook<S> {
        self.workbook
    }
}

impl<S: StorageEngine> Deref for Transaction<'_, S> {
    type Target = Workbook<S>;

    fn deref(&self) -> &Self::Target {
        self.workbook
    }
}

impl<S: StorageEngine> DerefMut for Transaction<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.workbook
    }
}

impl<S: StorageEngine> Drop for Transaction<'_, S> {
    fn drop(&mut self) {
        if self.open {
            *self.workbook.catalog_mut() = self.snapshot.clone();
        }
    }
}
