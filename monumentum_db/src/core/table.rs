use crate::core::row::Row;
use crate::core::schema::table_schema::TableSchema;
use crate::core::value::Value;
use crate::error::DbError;

#[derive(Debug, Clone)]
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
}

impl Table {
    #[must_use]
    pub fn new(schema: TableSchema) -> Self {
        Self {
            schema,
            rows: Vec::new(),
        }
    }

    #[must_use]
    pub fn schema(&self) -> &TableSchema {
        &self.schema
    }

    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn insert(&mut self, row: Row) -> Result<(), DbError> {
        self.schema.validate_values(row.values())?;

        let columns = self.schema.columns();
        for (idx, col) in columns.iter().enumerate() {
            if col.is_primary_key() || col.is_unique() {
                let new_val = row.get(idx).unwrap_or(&Value::Null);
                if self
                    .rows
                    .iter()
                    .any(|r| r.get(idx).is_some_and(|v| v == new_val))
                {
                    return Err(DbError::invalid_operation(format!(
                        "duplicate value for column '{}'",
                        col.name()
                    )));
                }
            }
        }

        self.rows.push(row);
        Ok(())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
}
