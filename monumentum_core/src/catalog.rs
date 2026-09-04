use crate::table::Table;
use alloc::collections::BTreeMap;
use monumentum_handler::{core::schema::table_schema::TableSchema, error::DbError};

#[derive(Debug, Default, Clone, PartialEq)]
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
        let _ = self.tables.insert(name, Table::new(schema));
        Ok(())
    }

    pub fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        if self.tables.remove(name).is_none() {
            return Err(DbError::table_not_found(name));
        }
        Ok(())
    }

    pub fn replace_table(&mut self, name: &str, table: Table) -> Result<(), DbError> {
        if !self.tables.contains_key(name) {
            return Err(DbError::table_not_found(name));
        }
        if table.schema().name() != name {
            return Err(DbError::invalid_operation(
                "table schema name does not match catalog key",
            ));
        }
        let _ = self.tables.insert(name.to_string(), table);
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

    pub fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        if old_name == new_name {
            return Ok(());
        }
        if self.tables.contains_key(new_name) {
            return Err(DbError::invalid_operation(format!(
                "table '{new_name}' already exists"
            )));
        }
        let mut table = self
            .tables
            .remove(old_name)
            .ok_or_else(|| DbError::table_not_found(old_name))?;
        table.rename_schema(new_name)?;
        let _ = self.tables.insert(new_name.to_string(), table);
        Ok(())
    }
}
