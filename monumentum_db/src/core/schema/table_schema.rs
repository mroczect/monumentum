use crate::core::schema::column::{ColumnDef, ComparisonOp, DataType};
use crate::core::value::Value;
use crate::error::DbError;
use std::collections::HashSet;

const MAX_COLUMNS: usize = 1024;

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

            if let Some(check) = col.check_constraint()
                && !evaluate_check(val, check)
            {
                return Err(DbError::invalid_operation(format!(
                    "check constraint failed for column '{}': {:?} {:?} {:?}",
                    col.name(),
                    check.column,
                    check.op,
                    check.value
                )));
            }
        }
        Ok(())
    }
}

fn evaluate_check(val: &Value, check: &crate::core::schema::column::CheckConstraint) -> bool {
    match (&val, &check.value) {
        (Value::Integer(a), Value::Integer(b)) => match check.op {
            ComparisonOp::Eq => a.as_i64() == b.as_i64(),
            ComparisonOp::NotEq => a.as_i64() != b.as_i64(),
            ComparisonOp::Lt => a.as_i64() < b.as_i64(),
            ComparisonOp::Lte => a.as_i64() <= b.as_i64(),
            ComparisonOp::Gt => a.as_i64() > b.as_i64(),
            ComparisonOp::Gte => a.as_i64() >= b.as_i64(),
        },
        (Value::Float(a), Value::Float(b)) => match check.op {
            ComparisonOp::Eq => a.as_f64() == b.as_f64(),
            ComparisonOp::NotEq => a.as_f64() != b.as_f64(),
            ComparisonOp::Lt => a.as_f64() < b.as_f64(),
            ComparisonOp::Lte => a.as_f64() <= b.as_f64(),
            ComparisonOp::Gt => a.as_f64() > b.as_f64(),
            ComparisonOp::Gte => a.as_f64() >= b.as_f64(),
        },
        (Value::Text(a), Value::Text(b)) => match check.op {
            ComparisonOp::Eq => a.as_str() == b.as_str(),
            ComparisonOp::NotEq => a.as_str() != b.as_str(),
            ComparisonOp::Lt => a.as_str() < b.as_str(),
            ComparisonOp::Lte => a.as_str() <= b.as_str(),
            ComparisonOp::Gt => a.as_str() > b.as_str(),
            ComparisonOp::Gte => a.as_str() >= b.as_str(),
        },
        (Value::Blob(a), Value::Blob(b)) => match check.op {
            ComparisonOp::Eq => a.as_slice() == b.as_slice(),
            ComparisonOp::NotEq => a.as_slice() != b.as_slice(),
            _ => false,
        },
        _ => false,
    }
}
