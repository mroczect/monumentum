use crate::index::{HashIndex, IndexKey};
use monumentum_handler::error::ErrorKind;
use monumentum_handler::{
    ColumnDef, core::row::Row, core::schema::table_schema::TableSchema, core::value::Value,
    error::DbError,
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
    unique_indexes: Vec<Option<HashIndex>>,
    read_only: bool,
}

impl Table {
    #[must_use]
    pub fn new(schema: TableSchema) -> Self {
        let mut unique_indexes = Vec::with_capacity(schema.columns().len());
        for col in schema.columns() {
            if col.is_unique() || col.is_primary_key() {
                unique_indexes.push(Some(HashIndex::new()));
            } else {
                unique_indexes.push(None);
            }
        }
        Self {
            schema,
            rows: Vec::new(),
            unique_indexes,
            read_only: false,
        }
    }

    pub fn rename_schema(&mut self, new_name: &str) -> Result<(), DbError> {
        let new_schema = TableSchema::try_new(new_name, self.schema.columns().to_vec())?;
        self.schema = new_schema;
        Ok(())
    }

    #[must_use]
    pub const fn schema(&self) -> &TableSchema {
        &self.schema
    }

    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn insert(&mut self, row: &Row) -> Result<(), DbError> {
        if self.read_only {
            return Err(DbError::invalid_operation("table is read-only"));
        }
        if row.len() != self.schema.columns().len() {
            return Err(DbError::invalid_operation(format!(
                "expected {} values, got {}",
                self.schema.columns().len(),
                row.len()
            )));
        }
        if self.rows.len() >= monumentum_handler::constants::MAX_ROWS_PER_TABLE {
            return Err(DbError::invalid_operation("too many rows"));
        }

        let mut values = row.values().to_vec();
        for (idx, col) in self.schema.columns().iter().enumerate() {
            if values
                .get(idx)
                .ok_or_else(|| DbError::invalid_operation("index out of bounds"))?
                .is_null()
                && let Some(default) = col.default_value()
            {
                *values
                    .get_mut(idx)
                    .ok_or_else(|| DbError::invalid_operation("index out of bounds"))? =
                    default.clone();
            }
        }
        let row = Row::new(values);

        self.schema.validate_values(row.values())?;

        for (idx, col) in self.schema.columns().iter().enumerate() {
            let is_unique_or_pk = col.is_unique() || col.is_primary_key();
            if is_unique_or_pk
                && let Some(val) = row.get(&idx)
                && !val.is_null()
                && let Some(key) = IndexKey::from_value(val)
                && let Some(index) = self.unique_indexes.get(idx).and_then(Option::as_ref)
                && index.contains(&key)
            {
                return Err(DbError::constraint_violation(
                    ErrorKind::UniqueViolation,
                    format!("duplicate value for column '{}'", col.name()),
                    Some(col.name().to_string()),
                    Some(self.schema.name().to_string()),
                ));
            }
        }

        let row_idx = self.rows.len();
        self.rows.push(row);

        for (idx, col) in self.schema.columns().iter().enumerate() {
            let is_unique_or_pk = col.is_unique() || col.is_primary_key();
            if is_unique_or_pk
                && let Some(val) = self.rows.get(row_idx).and_then(|r| r.get(&idx))
                && !val.is_null()
                && let Some(key) = IndexKey::from_value(val)
                && let Some(index) = self.unique_indexes.get_mut(idx).and_then(Option::as_mut)
            {
                index.insert(key, row_idx);
            }
        }

        Ok(())
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError> {
        if self.read_only {
            return Err(DbError::invalid_operation("table is read-only"));
        }

        for row in &rows {
            if row.len() != self.schema.columns().len() {
                return Err(DbError::invalid_operation(format!(
                    "expected {} values, got {}",
                    self.schema.columns().len(),
                    row.len()
                )));
            }
            self.schema.validate_values(row.values())?;
        }

        for (idx, col) in self.schema.columns().iter().enumerate() {
            if col.is_unique() || col.is_primary_key() {
                let mut seen_keys: HashSet<IndexKey> = HashSet::new();
                for row in &rows {
                    if let Some(val) = row.get(&idx)
                        && !val.is_null()
                        && let Some(key) = IndexKey::from_value(val)
                        && !seen_keys.insert(key)
                    {
                        return Err(DbError::constraint_violation(
                            ErrorKind::UniqueViolation,
                            format!("duplicate value for column '{}'", col.name()),
                            Some(col.name().to_string()),
                            Some(self.schema.name().to_string()),
                        ));
                    }
                }
            }
        }

        self.rows = rows;
        self.rebuild_indexes();
        Ok(())
    }

    pub fn set_cell(
        &mut self,
        row_idx: usize,
        col_idx: usize,
        value: &Value,
    ) -> Result<(), DbError> {
        if self.read_only {
            return Err(DbError::invalid_operation("table is read-only"));
        }
        if row_idx >= self.rows.len() || col_idx >= self.schema.columns().len() {
            return Err(DbError::invalid_operation("index out of bounds"));
        }

        let old_row = self
            .rows
            .get(row_idx)
            .ok_or_else(|| DbError::invalid_operation("row index out of bounds"))?;
        let mut new_values = old_row.values().to_vec();
        if let Some(cell) = new_values.get_mut(col_idx) {
            *cell = value.clone();
        } else {
            return Err(DbError::invalid_operation("column index out of bounds"));
        }
        let new_row = Row::new(new_values);
        self.schema.validate_values(new_row.values())?;

        if let Some(col) = self.schema.columns().get(col_idx) {
            if col.is_unique() || col.is_primary_key() {
                if !value.is_null()
                    && let Some(key) = IndexKey::from_value(value)
                {
                    let existing_indices = self
                        .unique_indexes
                        .get(col_idx)
                        .and_then(Option::as_ref)
                        .and_then(|index| index.get_indices(&key))
                        .map(<[usize]>::to_vec)
                        .unwrap_or_default();
                    if existing_indices.iter().any(|&r| r != row_idx) {
                        return Err(DbError::constraint_violation(
                            ErrorKind::UniqueViolation,
                            format!("duplicate value for column '{}'", col.name()),
                            Some(col.name().to_string()),
                            Some(self.schema.name().to_string()),
                        ));
                    }
                }

                if let Some(old_val) = old_row.get(&col_idx).cloned()
                    && !old_val.is_null()
                    && let Some(old_key) = IndexKey::from_value(&old_val)
                    && let Some(index) = self
                        .unique_indexes
                        .get_mut(col_idx)
                        .and_then(Option::as_mut)
                {
                    index.remove(&old_key, row_idx);
                }

                if let Some(row) = self.rows.get_mut(row_idx) {
                    *row = new_row;
                }

                if let Some(val) = self.rows.get(row_idx).and_then(|r| r.get(&col_idx))
                    && !val.is_null()
                    && let Some(new_key) = IndexKey::from_value(val)
                    && let Some(index) = self
                        .unique_indexes
                        .get_mut(col_idx)
                        .and_then(Option::as_mut)
                {
                    index.insert(new_key, row_idx);
                }
            } else if let Some(row) = self.rows.get_mut(row_idx) {
                *row = new_row;
            }
        } else {
            return Err(DbError::invalid_operation("column index out of bounds"));
        }

        Ok(())
    }

    pub fn set_column_allowed_values(
        &mut self,
        col_idx: usize,
        values: Option<&Vec<Value>>,
    ) -> Result<(), DbError> {
        if self.read_only {
            return Err(DbError::invalid_operation("table is read-only"));
        }

        let mut columns = self.schema.columns().to_vec();
        let col = columns
            .get_mut(col_idx)
            .ok_or_else(|| DbError::invalid_operation("column index out of bounds"))?;
        col.set_allowed_values(values.cloned());

        let new_schema = TableSchema::try_new(self.schema.name(), columns)?;

        for row in &self.rows {
            new_schema.validate_values(row.values())?;
        }

        self.schema = new_schema;
        Ok(())
    }

    fn rebuild_indexes(&mut self) {
        for idx in self.unique_indexes.iter_mut().flatten() {
            idx.clear();
        }

        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col_idx, col) in self.schema.columns().iter().enumerate() {
                let is_unique_or_pk = col.is_unique() || col.is_primary_key();
                if is_unique_or_pk
                    && let Some(val) = row.get(&col_idx)
                    && !val.is_null()
                    && let Some(key) = IndexKey::from_value(val)
                    && let Some(index) = self
                        .unique_indexes
                        .get_mut(col_idx)
                        .and_then(Option::as_mut)
                {
                    index.insert(key, row_idx);
                }
            }
        }
    }

    #[must_use]
    pub fn lookup_by_unique(&self, col_idx: usize, value: &Value) -> Option<&Row> {
        if let Some(index) = self.unique_indexes.get(col_idx).and_then(Option::as_ref)
            && let Some(key) = IndexKey::from_value(value)
            && let Some(indices) = index.get_indices(&key)
            && let Some(&row_idx) = indices.first()
        {
            return self.rows.get(row_idx);
        }
        self.rows
            .iter()
            .find(|row| row.get(&col_idx) == Some(value))
    }

    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub const fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
    }

    #[must_use]
    pub fn get_column_by_name(&self, name: &str) -> Option<&ColumnDef> {
        self.schema.get_column(name)
    }
}
