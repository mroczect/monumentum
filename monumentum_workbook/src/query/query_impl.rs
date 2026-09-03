use crate::{Workbook, WorkbookError};
use core::cmp::Ordering;
use core::marker::PhantomData;
use monumentum_db::core::row::Row;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
#[allow(missing_debug_implementations)]
pub struct Query<'a, S: StorageEngine, F> {
    workbook: &'a Workbook<S>,
    sheet: String,
    columns: Option<Vec<usize>>,
    filter: Option<F>,
    sort_by: Option<(usize, bool)>,
    limit: Option<usize>,
    _marker: PhantomData<F>,
}

impl<'a, S: StorageEngine, F> Query<'a, S, F>
where
    F: Fn(&Row) -> bool,
{
    pub fn new(workbook: &'a Workbook<S>, sheet: impl Into<String>) -> Self {
        Self {
            workbook,
            sheet: sheet.into(),
            columns: None,
            filter: None,
            sort_by: None,
            limit: None,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn select(mut self, columns: Vec<usize>) -> Self {
        self.columns = Some(columns);
        self
    }

    #[must_use]
    pub fn filter(mut self, predicate: F) -> Self {
        self.filter = Some(predicate);
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

    pub fn fetch_all(self) -> Result<Vec<Row>, WorkbookError> {
        let table = self.workbook.sheet(&self.sheet)?;
        let mut rows: Vec<Row> = table
            .rows()
            .iter()
            .filter(|row| self.filter.as_ref().is_none_or(|f| f(row)))
            .cloned()
            .collect();

        if let Some((col, asc)) = self.sort_by {
            rows.sort_by(|a, b| {
                let va = a.get(col).unwrap_or(&Value::Null);
                let vb = b.get(col).unwrap_or(&Value::Null);
                let ord = va.partial_cmp(vb).unwrap_or(Ordering::Equal);
                if asc { ord } else { ord.reverse() }
            });
        }

        if let Some(limit) = self.limit {
            rows.truncate(limit);
        }

        if let Some(cols) = &self.columns {
            rows = rows
                .into_iter()
                .map(|row| {
                    let vals: Vec<Value> =
                        cols.iter().filter_map(|&i| row.get(i).cloned()).collect();
                    Row::new(vals)
                })
                .collect();
        }

        Ok(rows)
    }

    pub fn fetch_one(self) -> Result<Row, WorkbookError> {
        self.fetch_optional()?.ok_or(WorkbookError::Null)
    }

    pub fn fetch_optional(self) -> Result<Option<Row>, WorkbookError> {
        let mut rows = self.fetch_all()?;
        Ok(if rows.is_empty() {
            None
        } else {
            Some(rows.remove(0))
        })
    }

    pub fn map<G, O>(self, mut f: G) -> Map<'a, S, impl FnMut(Row) -> Result<O, WorkbookError>, F>
    where
        G: FnMut(Row) -> O,
    {
        self.try_map(move |row| Ok(f(row)))
    }

    #[must_use]
    pub const fn try_map<G, O>(self, f: G) -> Map<'a, S, G, F>
    where
        G: FnMut(Row) -> Result<O, WorkbookError>,
    {
        Map {
            inner: self,
            mapper: f,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct Map<'a, S: StorageEngine, F, A> {
    inner: Query<'a, S, A>,
    mapper: F,
}

impl<'a, S: StorageEngine, F, O, A> Map<'a, S, F, A>
where
    F: FnMut(Row) -> Result<O, WorkbookError>,
    A: Fn(&Row) -> bool,
{
    pub fn fetch_all(mut self) -> Result<Vec<O>, WorkbookError> {
        let rows = self.inner.fetch_all()?;
        rows.into_iter().map(|row| (self.mapper)(row)).collect()
    }

    pub fn fetch_one(self) -> Result<O, WorkbookError> {
        self.fetch_optional()?.ok_or(WorkbookError::Null)
    }

    pub fn fetch_optional(mut self) -> Result<Option<O>, WorkbookError> {
        let row = self.inner.fetch_optional()?;
        row.map(|r| (self.mapper)(r)).transpose()
    }

    pub fn map<G, P>(self, mut g: G) -> Map<'a, S, impl FnMut(Row) -> Result<P, WorkbookError>, A>
    where
        G: FnMut(O) -> P,
    {
        self.try_map(move |data| Ok(g(data)))
    }

    pub fn try_map<G, P>(
        self,
        mut g: G,
    ) -> Map<'a, S, impl FnMut(Row) -> Result<P, WorkbookError>, A>
    where
        G: FnMut(O) -> Result<P, WorkbookError>,
    {
        let mut f = self.mapper;
        Map {
            inner: self.inner,
            mapper: move |row| f(row).and_then(&mut g),
        }
    }
}
