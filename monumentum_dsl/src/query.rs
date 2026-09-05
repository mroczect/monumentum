use core::cmp::Ordering;
use core::fmt;
use monumentum_handler::core::row::Row;
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
