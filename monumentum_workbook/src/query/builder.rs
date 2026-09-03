use crate::Workbook;
use crate::query::{FromRow, Query, QueryAs};
use core::marker::PhantomData;
use monumentum_db::core::row::Row;
use monumentum_db::store::storage::StorageEngine;

type Filter<'a> = Box<dyn Fn(&Row) -> bool + 'a>;
#[allow(missing_debug_implementations)]
pub struct QueryBuilder<'a, S: StorageEngine> {
    workbook: &'a Workbook<S>,
    sheet: String,
    columns: Option<Vec<usize>>,
    filters: Vec<Filter<'a>>,
    sort_by: Option<(usize, bool)>,
    limit: Option<usize>,
}

impl<'a, S: StorageEngine> QueryBuilder<'a, S> {
    pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self {
        Self {
            workbook,
            sheet: sheet.into(),
            columns: None,
            filters: Vec::new(),
            sort_by: None,
            limit: None,
        }
    }

    #[must_use]
    pub fn select(mut self, columns: Vec<usize>) -> Self {
        self.columns = Some(columns);
        self
    }

    #[must_use]
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Row) -> bool + 'a,
    {
        self.filters.push(Box::new(predicate));
        self
    }

    #[must_use]
    pub const fn order_by(mut self, col: usize, ascending: bool) -> Self {
        self.sort_by = Some((col, ascending));
        self
    }

    #[must_use]
    pub const fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    #[must_use]
    pub fn build(self) -> Query<'a, S, impl Fn(&Row) -> bool> {
        let filters = self.filters;
        let combined_filter = move |row: &Row| filters.iter().all(|f| f(row));

        let mut query = Query::new(self.workbook, self.sheet).filter(combined_filter);
        if let Some(cols) = self.columns {
            query = query.select(cols);
        }
        if let Some((col, asc)) = self.sort_by {
            query = query.order_by(col, asc);
        }
        if let Some(limit) = self.limit {
            query = query.limit(limit);
        }
        query
    }

    #[must_use]
    pub fn build_query_as<O>(self) -> QueryAs<'a, S, O, impl Fn(&Row) -> bool>
    where
        O: FromRow,
    {
        QueryAs {
            inner: self.build(),
            _output: PhantomData,
        }
    }
}
