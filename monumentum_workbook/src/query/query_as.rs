use crate::query::FromRow;
use crate::query::query_impl::Query;
use crate::{Workbook, WorkbookError};
use core::marker::PhantomData;
use monumentum_db::core::row::Row;
use monumentum_db::store::storage::StorageEngine;

#[allow(missing_debug_implementations)]
pub struct QueryAs<'a, S: StorageEngine, O> {
    pub(crate) inner: Query<'a, S>,
    pub(crate) _output: PhantomData<O>,
}

impl<'a, S: StorageEngine, O> QueryAs<'a, S, O>
where
    O: FromRow,
{
    pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self {
        Self {
            inner: Query::new(workbook, sheet),
            _output: PhantomData,
        }
    }

    #[must_use]
    pub fn select(mut self, columns: Vec<usize>) -> Self {
        self.inner = self.inner.select(columns);
        self
    }

    #[must_use]
    pub fn filter(mut self, predicate: impl Fn(&Row) -> bool + 'a) -> Self {
        self.inner = self.inner.filter(predicate);
        self
    }

    #[must_use]
    pub fn order_by(mut self, col: usize, ascending: bool) -> Self {
        self.inner = self.inner.order_by(col, ascending);
        self
    }

    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.inner = self.inner.limit(n);
        self
    }

    pub fn fetch_all(self) -> Result<Vec<O>, WorkbookError> {
        let rows = self.inner.fetch_all()?;
        rows.iter().map(O::from_row).collect()
    }

    pub fn fetch_one(self) -> Result<O, WorkbookError> {
        self.fetch_optional()?.ok_or(WorkbookError::Null)
    }

    pub fn fetch_optional(self) -> Result<Option<O>, WorkbookError> {
        let row = self.inner.fetch_optional()?;
        row.map(|r| O::from_row(&r)).transpose()
    }
}
