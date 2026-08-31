use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use crate::error::DbError;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct Catalog {
    tables: BTreeMap<String, Table>,
}
impl Catalog {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        let name = schema.name().to_string();
        if name.is_empty() {
            return Err(DbError::invalid_operation("table name cannot be empty"));
        }
        if self.tables.contains_key(&name) {
            return Err(DbError::invalid_operation(format!(
                "table '{name}' already exists"
            )));
        }
        self.tables.insert(name, Table::new(schema));
        Ok(())
    }

    pub fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        if self.tables.remove(name).is_none() {
            return Err(DbError::table_not_found(name));
        }
        Ok(())
    }

    #[must_use]
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    #[must_use]
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }

    pub fn tables(&self) -> impl Iterator<Item = (&str, &Table)> {
        self.tables.iter().map(|(k, v)| (k.as_str(), v))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }
}
