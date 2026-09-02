use crate::core::index::HashIndex;
use crate::core::index::IndexKey;
use crate::core::row::Row;
use crate::core::schema::table_schema::TableSchema;
use crate::core::value::Value;
use crate::error::DbError;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
    unique_indexes: Vec<Option<HashIndex>>,
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
        }
    }

    #[must_use]
    pub fn schema(&self) -> &TableSchema {
        &self.schema
    }

    #[must_use]
    pub fn schema_mut(&mut self) -> &mut TableSchema {
        &mut self.schema
    }

    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }

    #[must_use]
    pub fn rows_mut(&mut self) -> &mut Vec<Row> {
        &mut self.rows
    }

    pub fn insert(&mut self, row: Row) -> Result<(), DbError> {
        if row.len() != self.schema.columns().len() {
            return Err(DbError::invalid_operation(format!(
                "expected {} values, got {}",
                self.schema.columns().len(),
                row.len()
            )));
        }

        let mut values = row.values().to_vec();
        for (idx, col) in self.schema.columns().iter().enumerate() {
            if values[idx].is_null()
                && let Some(default) = col.default_value()
            {
                values[idx] = default.clone();
            }
        }
        let row = Row::new(values);

        self.schema.validate_values(row.values())?;

        for (idx, col) in self.schema.columns().iter().enumerate() {
            if col.is_unique() || col.is_primary_key() {
                let val = row.get(idx).unwrap_or(&Value::Null);
                if !val.is_null()
                    && let Some(key) = IndexKey::from_value(val)
                    && let Some(index) = &self.unique_indexes[idx]
                    && index.contains(&key)
                {
                    return Err(DbError::invalid_operation(format!(
                        "duplicate value for column '{}'",
                        col.name()
                    )));
                }
            }
        }

        let row_idx = self.rows.len();
        self.rows.push(row);

        for (idx, col) in self.schema.columns().iter().enumerate() {
            if col.is_unique() || col.is_primary_key() {
                let val = self.rows[row_idx].get(idx).unwrap_or(&Value::Null);
                if !val.is_null()
                    && let (Some(key), Some(index)) =
                        (IndexKey::from_value(val), &mut self.unique_indexes[idx])
                {
                    index.insert(key, row_idx);
                }
            }
        }

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

    pub fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError> {
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
                let mut seen_keys: Vec<IndexKey> = Vec::new();
                for row in &rows {
                    let val = row.get(idx).unwrap_or(&Value::Null);
                    if !val.is_null()
                        && let Some(key) = IndexKey::from_value(val)
                    {
                        if seen_keys.contains(&key) {
                            return Err(DbError::invalid_operation(format!(
                                "duplicate value for column '{}'",
                                col.name()
                            )));
                        }
                        seen_keys.push(key);
                    }
                }
            }
        }

        self.rows = rows;
        self.rebuild_indexes();
        Ok(())
    }

    fn rebuild_indexes(&mut self) {
        for idx in self.unique_indexes.iter_mut().flatten() {
            idx.clear();
        }

        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col_idx, col) in self.schema.columns().iter().enumerate() {
                if col.is_unique() || col.is_primary_key() {
                    let val = row.get(col_idx).unwrap_or(&Value::Null);
                    if !val.is_null()
                        && let (Some(key), Some(index)) =
                            (IndexKey::from_value(val), &mut self.unique_indexes[col_idx])
                    {
                        index.insert(key, row_idx);
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn lookup_by_unique(&self, col_idx: usize, value: &Value) -> Option<&Row> {
        if let Some(Some(index)) = self.unique_indexes.get(col_idx)
            && let Some(key) = IndexKey::from_value(value)
        {
            if let Some(indices) = index.get_indices(&key)
                && let Some(&row_idx) = indices.first()
            {
                return self.rows.get(row_idx);
            }
            return None;
        }
        self.rows.iter().find(|row| row.get(col_idx) == Some(value))
    }
}
