use crate::{Workbook, WorkbookError};
use core::cmp::Ordering;
use core::marker::PhantomData;
use monumentum_db::core::row::Row;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
use monumentum_db::types::{Float, Integer};
use monumentum_query::formula::FormulaError;

type Filter<'a> = Box<dyn Fn(&Row) -> bool + 'a>;

#[allow(missing_debug_implementations)]
pub struct Query<'a, S: StorageEngine> {
    workbook: &'a Workbook<S>,
    sheet: String,
    columns: Option<Vec<usize>>,
    filter: Option<Filter<'a>>,
    sort_by: Option<(usize, bool)>,
    limit: Option<usize>,
    _marker: PhantomData<S>,
}

impl<'a, S: StorageEngine> Query<'a, S> {
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
    pub fn select_by_names(mut self, columns: &[&str]) -> Self {
        if let Ok(schema) = self.workbook.sheet(&self.sheet).map(Table::schema) {
            let indices: Vec<usize> = columns
                .iter()
                .filter_map(|name| schema.column_index(name))
                .collect();
            self.columns = Some(indices);
        } else {
            self.columns = Some(Vec::new());
        }
        self
    }

    #[must_use]
    pub fn filter(mut self, predicate: impl Fn(&Row) -> bool + 'a) -> Self {
        self.filter = Some(Box::new(predicate));
        self
    }

    #[must_use]
    pub fn filter_by_column<F>(mut self, col_name: &str, predicate: F) -> Self
    where
        F: Fn(&Value) -> bool + 'a,
    {
        if let Ok(schema) = self.workbook.sheet(&self.sheet).map(Table::schema)
            && let Some(idx) = schema.column_index(col_name)
        {
            let existing = self.filter.take();
            let new_filter = move |row: &Row| {
                row.get(idx).is_some_and(&predicate) && existing.as_ref().is_none_or(|f| f(row))
            };
            self.filter = Some(Box::new(new_filter));
        }
        self
    }

    #[must_use]
    pub const fn order_by(mut self, col: usize, ascending: bool) -> Self {
        self.sort_by = Some((col, ascending));
        self
    }

    #[must_use]
    pub fn order_by_name(mut self, col_name: &str, ascending: bool) -> Self {
        if let Ok(schema) = self.workbook.sheet(&self.sheet).map(Table::schema)
            && let Some(idx) = schema.column_index(col_name)
        {
            self.sort_by = Some((idx, ascending));
        }
        self
    }

    #[must_use]
    pub const fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn count(self) -> Result<usize, WorkbookError> {
        self.fetch_all().map(|rows| rows.len())
    }

    pub fn sum(self, col: usize) -> Result<Value, WorkbookError> {
        let rows = self.fetch_all()?;
        let mut sum_int: i64 = 0;
        let mut sum_float: f64 = 0.0;
        let mut has_float = false;

        for row in &rows {
            let val = row.get(col).ok_or(WorkbookError::InvalidReference)?;
            match val {
                Value::Integer(i) => {
                    if has_float {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            sum_float += i.as_i64() as f64;
                        }
                    } else {
                        sum_int = sum_int.checked_add(i.as_i64()).ok_or_else(|| {
                            WorkbookError::Formula(FormulaError::Eval(
                                "integer overflow".to_string(),
                            ))
                        })?;
                    }
                }
                Value::Float(f) => {
                    if !has_float {
                        has_float = true;
                        #[allow(clippy::cast_precision_loss)]
                        {
                            sum_float = sum_int as f64;
                        }
                    }
                    sum_float += f.as_f64();
                    if !sum_float.is_finite() {
                        return Err(WorkbookError::Formula(FormulaError::Eval(
                            "float overflow".to_string(),
                        )));
                    }
                }
                Value::Null
                | Value::Text(_)
                | Value::Blob(_)
                | Value::Boolean(_)
                | Value::Formula(_)
                | _ => {
                    return Err(WorkbookError::Formula(FormulaError::TypeMismatch(
                        "SUM expects numeric values".to_string(),
                    )));
                }
            }
        }

        if has_float {
            Float::try_new(sum_float)
                .map(Value::Float)
                .map_err(|e| WorkbookError::Formula(FormulaError::Eval(e.to_string())))
        } else {
            Ok(Value::Integer(Integer::new(sum_int)))
        }
    }

    pub fn avg(self, col: usize) -> Result<Value, WorkbookError> {
        let rows = self.fetch_all()?;
        if rows.is_empty() {
            return Err(WorkbookError::Formula(FormulaError::Eval(
                "AVG of empty set".to_string(),
            )));
        }
        let mut sum = 0.0_f64;
        let mut count: usize = 0;
        for row in &rows {
            let val = row.get(col).ok_or(WorkbookError::InvalidReference)?;
            match val {
                Value::Integer(i) => {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        sum += i.as_i64() as f64;
                    }
                }
                Value::Float(f) => sum += f.as_f64(),
                Value::Null
                | Value::Text(_)
                | Value::Blob(_)
                | Value::Boolean(_)
                | Value::Formula(_)
                | _ => {
                    return Err(WorkbookError::Formula(FormulaError::TypeMismatch(
                        "AVG expects numeric values".to_string(),
                    )));
                }
            }
            count = count.saturating_add(1);
        }
        #[allow(clippy::cast_precision_loss)]
        let avg = sum / count as f64;
        Float::try_new(avg)
            .map(Value::Float)
            .map_err(|e| WorkbookError::Formula(FormulaError::Eval(e.to_string())))
    }

    pub fn min(self, col: usize) -> Result<Value, WorkbookError> {
        let rows = self.fetch_all()?;
        let mut min_val: Option<Value> = None;
        for row in &rows {
            let val = row.get(col).ok_or(WorkbookError::InvalidReference)?;
            if min_val.as_ref().is_none_or(|m| val < m) {
                min_val = Some(val.clone());
            }
        }
        min_val.ok_or(WorkbookError::Null)
    }

    pub fn max(self, col: usize) -> Result<Value, WorkbookError> {
        let rows = self.fetch_all()?;
        let mut max_val: Option<Value> = None;
        for row in &rows {
            let val = row.get(col).ok_or(WorkbookError::InvalidReference)?;
            if max_val.as_ref().is_none_or(|m| val > m) {
                max_val = Some(val.clone());
            }
        }
        max_val.ok_or(WorkbookError::Null)
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

    pub fn map<G, O>(self, mut f: G) -> Map<'a, S, impl FnMut(Row) -> Result<O, WorkbookError>>
    where
        G: FnMut(Row) -> O,
    {
        self.try_map(move |row| Ok(f(row)))
    }

    #[must_use]
    pub const fn try_map<G, O>(self, f: G) -> Map<'a, S, G>
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
pub struct Map<'a, S: StorageEngine, F> {
    inner: Query<'a, S>,
    mapper: F,
}

impl<'a, S: StorageEngine, F, O> Map<'a, S, F>
where
    F: FnMut(Row) -> Result<O, WorkbookError>,
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

    pub fn map<G, P>(self, mut g: G) -> Map<'a, S, impl FnMut(Row) -> Result<P, WorkbookError>>
    where
        G: FnMut(O) -> P,
    {
        self.try_map(move |data| Ok(g(data)))
    }

    pub fn try_map<G, P>(self, mut g: G) -> Map<'a, S, impl FnMut(Row) -> Result<P, WorkbookError>>
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
