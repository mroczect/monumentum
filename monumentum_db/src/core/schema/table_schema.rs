use crate::core::schema::column::ColumnDef;
use crate::core::value::Value;
use crate::error::DbError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnDef>,
}

impl TableSchema {
    #[must_use]
    pub fn new(name: impl Into<String>, columns: Vec<ColumnDef>) -> Self {
        Self {
            name: name.into(),
            columns,
        }
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
            let type_ok = match col.data_type() {
                crate::core::schema::column::DataType::Null => true,
                crate::core::schema::column::DataType::Integer => val.is_integer(),
                crate::core::schema::column::DataType::Float => val.is_float(),
                crate::core::schema::column::DataType::Text => val.is_text(),
                crate::core::schema::column::DataType::Blob => val.is_blob(),
            };
            if !type_ok {
                return Err(DbError::type_mismatch(format!(
                    "column '{}' expects {}, got {}",
                    col.name(),
                    col.data_type(),
                    val.type_name()
                )));
            }
            if !col.is_nullable() && val.is_null() {
                return Err(DbError::invalid_operation(format!(
                    "column '{}' is not nullable",
                    col.name()
                )));
            }
        }
        Ok(())
    }
}
