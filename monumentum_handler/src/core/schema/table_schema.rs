use crate::constants::{MAX_COLUMNS, MAX_NAME_LENGTH};
use crate::core::schema::column::ColumnDef;
use crate::core::value::Value;
use crate::error::DbError;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
}

impl TableSchema {
    pub fn try_new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Result<Self, DbError> {
        let name = name.into();
        if name.is_empty() {
            return Err(DbError::invalid_operation("table name cannot be empty"));
        }
        if name.len() > MAX_NAME_LENGTH {
            return Err(DbError::invalid_operation(format!(
                "table name too long: {} bytes (max {})",
                name.len(),
                MAX_NAME_LENGTH
            )));
        }
        if name.chars().any(char::is_control) {
            return Err(DbError::invalid_operation(
                "table name contains control characters",
            ));
        }

        if columns.is_empty() {
            return Err(DbError::invalid_operation(
                "table must have at least one column",
            ));
        }
        if columns.len() > MAX_COLUMNS {
            return Err(DbError::invalid_operation(format!(
                "too many columns: {} (max {})",
                columns.len(),
                MAX_COLUMNS
            )));
        }

        let mut seen = HashSet::new();
        for col in &columns {
            let col_name = col.name();
            if col_name.is_empty() {
                return Err(DbError::invalid_operation("column name cannot be empty"));
            }
            if col_name.len() > MAX_NAME_LENGTH {
                return Err(DbError::invalid_operation(format!(
                    "column name too long: {} bytes (max {})",
                    col_name.len(),
                    MAX_NAME_LENGTH
                )));
            }
            if col_name.chars().any(char::is_control) {
                return Err(DbError::invalid_operation(
                    "column name contains control characters",
                ));
            }
            if !seen.insert(col_name.to_lowercase()) {
                return Err(DbError::invalid_operation(format!(
                    "duplicate column name '{}'",
                    col_name
                )));
            }
        }

        Ok(Self { name, columns })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
    }

    #[must_use]
    pub fn get_column_mut(&mut self, index: usize) -> Option<&mut ColumnDef> {
        self.columns.get_mut(index)
    }

    #[must_use]
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.name().eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.column_index(name)
            .and_then(|idx| self.columns.get(idx))
    }

    pub fn validate_values(&self, values: &[Value]) -> Result<(), DbError> {
        if values.len() != self.columns.len() {
            return Err(DbError::invalid_operation(format!(
                "expected {} values, got {}",
                self.columns.len(),
                values.len()
            )));
        }
        for (col, val) in self.columns.iter().zip(values) {
            col.validate_value(val)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn get_column_by_index<I>(&self, index: &I) -> Option<&ColumnDef>
    where
        I: crate::core::schema::column::ColumnIndex<Self>,
    {
        index.index(self).ok().and_then(|i| self.columns.get(i))
    }
}
