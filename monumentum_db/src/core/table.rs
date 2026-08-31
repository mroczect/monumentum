use crate::core::row::Row;
use crate::core::schema::table_schema::TableSchema;
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
