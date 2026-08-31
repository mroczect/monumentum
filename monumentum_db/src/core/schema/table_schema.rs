use crate::core::schema::column::{ColumnDef, DataType};
use crate::core::value::Value;
use crate::error::DbError;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
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

        if columns.is_empty() {
            return Err(DbError::invalid_operation(
                "table must have at least one column",
            ));
        }

        let mut seen = HashSet::new();
        for col in &columns {
            let col_name = col.name();
            if col_name.is_empty() {
                return Err(DbError::invalid_operation("column name cannot be empty"));
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
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.name().eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.column_index(name).map(|idx| &self.columns[idx])
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
            if val.is_null() {
                if !col.is_nullable() {
                    return Err(DbError::invalid_operation(format!(
                        "column '{}' is not nullable",
                        col.name()
                    )));
                }
                continue;
            }

            let type_ok = match col.data_type() {
                DataType::Null => false,
                DataType::Integer => val.is_integer(),
                DataType::Float => val.is_float(),
                DataType::Text => val.is_text(),
                DataType::Blob => val.is_blob(),
            };
            if !type_ok {
                return Err(DbError::type_mismatch(format!(
                    "column '{}' expects {}, got {}",
                    col.name(),
                    col.data_type(),
                    val.type_name()
                )));
            }
        }
        Ok(())
    }
}
