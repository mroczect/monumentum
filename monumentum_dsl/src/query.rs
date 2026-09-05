use core::cmp::Ordering;
use core::fmt;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;

type RowFilter<'a> = Box<dyn Fn(&Row) -> Result<bool, DbError> + 'a>;
type RowSort<'a> = Box<dyn Fn(&Row, &Row) -> Ordering + 'a>;
type ValueFilter<'a, T> = Box<dyn Fn(&T) -> Result<bool, DbError> + 'a>;
type ValueSort<'a, T> = Box<dyn Fn(&T, &T) -> Ordering + 'a>;

pub struct QueryBuilder<'a> {
    table: String,
    storage: &'a mut dyn StorageEngine,
    operations: Vec<RowOperation<'a>>,
}

impl fmt::Debug for QueryBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryBuilder")
            .field("table", &self.table)
            .field("operations", &self.operations.len())
            .finish()
    }
}

enum RowOperation<'a> {
    Filter(RowFilter<'a>),
    Sort(RowSort<'a>),
    Limit(usize),
    Offset(usize),
}

impl fmt::Debug for RowOperation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Filter(_) => write!(f, "Filter(..)"),
            Self::Sort(_) => write!(f, "Sort(..)"),
            Self::Limit(n) => f.debug_tuple("Limit").field(n).finish(),
            Self::Offset(n) => f.debug_tuple("Offset").field(n).finish(),
        }
    }
}

impl<'a> QueryBuilder<'a> {
    pub fn new(storage: &'a mut dyn StorageEngine, table: &str) -> Self {
        Self {
            table: table.to_string(),
            storage,
            operations: Vec::new(),
        }
    }

    #[must_use]
    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&Row) -> Result<bool, DbError> + 'a,
    {
        self.operations.push(RowOperation::Filter(Box::new(f)));
        self
    }

    #[must_use]
    pub fn sort_by<F>(mut self, f: F) -> Self
    where
        F: Fn(&Row, &Row) -> Ordering + 'a,
    {
        self.operations.push(RowOperation::Sort(Box::new(f)));
        self
    }

    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.operations.push(RowOperation::Limit(n));
        self
    }

    #[must_use]
    pub fn offset(mut self, n: usize) -> Self {
        self.operations.push(RowOperation::Offset(n));
        self
    }

    pub fn project<F, T>(self, f: F) -> Result<ProjectedQueryBuilder<'a, T>, DbError>
    where
        F: Fn(&Row) -> Result<T, DbError> + 'a,
        T: 'a,
    {
        let rows = self.collect_rows()?;
        let projected = rows.iter().map(f).collect::<Result<Vec<_>, _>>()?;
        Ok(ProjectedQueryBuilder {
            items: projected,
            operations: Vec::new(),
        })
    }

    pub fn aggregate<F>(
        self,
        aggregate: &dyn crate::functions::AggregateFunction,
        extractor: F,
    ) -> Result<Value, DbError>
    where
        F: Fn(&Row) -> Result<Value, DbError> + 'a,
    {
        let rows = self.collect_rows()?;
        let mut acc = aggregate.init();
        for row in &rows {
            let value = extractor(row)?;
            acc.update(&value)?;
        }
        acc.finish()
    }

    pub fn aggregate_by_name<F>(self, name: &str, extractor: F) -> Result<Value, DbError>
    where
        F: Fn(&Row) -> Result<Value, DbError> + 'a,
    {
        let registry = crate::functions::FunctionRegistry::new();
        let agg = registry.get_aggregate(name).ok_or_else(|| {
            DbError::unsupported(format!("aggregate function '{name}' not found"))
        })?;
        self.aggregate(agg, extractor)
    }
    pub fn group_by<F>(self, key_fn: F) -> Result<Vec<(Value, Vec<Row>)>, DbError>
    where
        F: Fn(&Row) -> Result<Value, DbError> + 'a,
    {
        let rows = self.collect_rows()?;
        let mut groups: Vec<(Value, Vec<Row>)> = Vec::new();

        for row in rows {
            let key = key_fn(&row)?;
            if let Some((_, group_rows)) = groups
                .iter_mut()
                .find(|(existing_key, _)| *existing_key == key)
            {
                group_rows.push(row);
            } else {
                groups.push((key, alloc::vec![row]));
            }
        }

        Ok(groups)
    }

    pub fn join_inner<LF, RF>(
        self,
        right_table: &str,
        left_key_fn: LF,
        right_key_fn: RF,
    ) -> Result<Vec<Row>, DbError>
    where
        LF: Fn(&Row) -> Result<Value, DbError> + 'a,
        RF: Fn(&Row) -> Result<Value, DbError> + 'a,
    {
        let right_rows = self.storage.get_all_rows(right_table)?;
        let left_rows = self.collect_rows()?;

        let mut result = Vec::new();

        for left in &left_rows {
            let left_key = left_key_fn(left)?;
            for right in &right_rows {
                let right_key = right_key_fn(right)?;
                if left_key == right_key {
                    let mut combined_values = left.values().to_vec();
                    combined_values.extend_from_slice(right.values());
                    result.push(Row::new(combined_values));
                }
            }
        }

        Ok(result)
    }

    pub fn join_left<LF, RF>(
        self,
        right_table: &str,
        left_key_fn: LF,
        right_key_fn: RF,
    ) -> Result<Vec<Row>, DbError>
    where
        LF: Fn(&Row) -> Result<Value, DbError> + 'a,
        RF: Fn(&Row) -> Result<Value, DbError> + 'a,
    {
        let right_rows = self.storage.get_all_rows(right_table)?;
        let left_rows = self.collect_rows()?;

        let mut result = Vec::new();

        for left in &left_rows {
            let left_key = left_key_fn(left)?;
            let mut found_match = false;

            for right in &right_rows {
                let right_key = right_key_fn(right)?;
                if left_key == right_key {
                    let mut combined_values = left.values().to_vec();
                    combined_values.extend_from_slice(right.values());
                    result.push(Row::new(combined_values));
                    found_match = true;
                }
            }

            if !found_match {
                let mut combined_values = left.values().to_vec();
                let extra_len = right_rows.first().map_or(0, Row::len);
                let new_len = combined_values.len().saturating_add(extra_len);
                combined_values.resize(new_len, Value::Null);
                result.push(Row::new(combined_values));
            }
        }

        Ok(result)
    }

    pub fn execute(self) -> Result<Vec<Row>, DbError> {
        self.collect_rows()
    }

    fn collect_rows(self) -> Result<Vec<Row>, DbError> {
        let mut rows = self.storage.get_all_rows(&self.table)?;

        for op in self.operations {
            match op {
                RowOperation::Filter(f) => {
                    let mut filtered = Vec::with_capacity(rows.len());
                    for row in rows {
                        if f(&row)? {
                            filtered.push(row);
                        }
                    }
                    rows = filtered;
                }
                RowOperation::Sort(cmp) => rows.sort_by(|a, b| cmp(a, b)),
                RowOperation::Limit(n) => rows.truncate(n),
                RowOperation::Offset(n) => {
                    if n >= rows.len() {
                        rows.clear();
                    } else {
                        let _ = rows.drain(0..n);
                    }
                }
            }
        }

        Ok(rows)
    }
}

pub struct ProjectedQueryBuilder<'a, T> {
    items: Vec<T>,
    operations: Vec<ProjectedOperation<'a, T>>,
}

impl<T> fmt::Debug for ProjectedQueryBuilder<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectedQueryBuilder")
            .field("items", &self.items.len())
            .field("operations", &self.operations.len())
            .finish()
    }
}

enum ProjectedOperation<'a, T> {
    Filter(ValueFilter<'a, T>),
    Sort(ValueSort<'a, T>),
    Limit(usize),
    Offset(usize),
}

impl<T> fmt::Debug for ProjectedOperation<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Filter(_) => write!(f, "Filter(..)"),
            Self::Sort(_) => write!(f, "Sort(..)"),
            Self::Limit(n) => f.debug_tuple("Limit").field(n).finish(),
            Self::Offset(n) => f.debug_tuple("Offset").field(n).finish(),
        }
    }
}

impl<'a, T> ProjectedQueryBuilder<'a, T> {
    #[must_use]
    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Result<bool, DbError> + 'a,
    {
        self.operations
            .push(ProjectedOperation::Filter(Box::new(f)));
        self
    }

    #[must_use]
    pub fn sort_by<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, &T) -> Ordering + 'a,
    {
        self.operations.push(ProjectedOperation::Sort(Box::new(f)));
        self
    }

    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.operations.push(ProjectedOperation::Limit(n));
        self
    }

    #[must_use]
    pub fn offset(mut self, n: usize) -> Self {
        self.operations.push(ProjectedOperation::Offset(n));
        self
    }

    pub fn execute(self) -> Result<Vec<T>, DbError> {
        let mut items = self.items;

        for op in self.operations {
            match op {
                ProjectedOperation::Filter(f) => {
                    let mut filtered = Vec::with_capacity(items.len());
                    for item in items {
                        if f(&item)? {
                            filtered.push(item);
                        }
                    }
                    items = filtered;
                }
                ProjectedOperation::Sort(cmp) => items.sort_by(|a, b| cmp(a, b)),
                ProjectedOperation::Limit(n) => items.truncate(n),
                ProjectedOperation::Offset(n) => {
                    if n >= items.len() {
                        items.clear();
                    } else {
                        let _ = items.drain(0..n);
                    }
                }
            }
        }

        Ok(items)
    }
}
