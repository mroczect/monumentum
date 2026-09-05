use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::{CatalogStore, Index, StorageEngine, TableStore};
use monumentum_handler::types::Text;
use proptest as _;
use std::collections::HashMap;
#[derive(Default)]
struct MockCatalog {
    tables: HashMap<String, TableSchema>,
}

impl CatalogStore for MockCatalog {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        let name = schema.name().to_string();
        if self.tables.contains_key(&name) {
            return Err(DbError::invalid_operation("table already exists"));
        }
        let _ = self.tables.insert(name, schema);
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        if self.tables.remove(name).is_none() {
            return Err(DbError::table_not_found(name));
        }
        Ok(())
    }

    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        if let Some(schema) = self.tables.remove(old_name) {
            let _ = self.tables.insert(new_name.to_string(), schema);
            Ok(())
        } else {
            Err(DbError::table_not_found(old_name))
        }
    }
}

#[derive(Default)]
struct MockIndex {
    entries: Vec<(Value, usize)>,
}

impl Index for MockIndex {
    fn insert(&mut self, key: &Value, row_idx: usize) {
        self.entries.push((key.clone(), row_idx));
    }

    fn remove(&mut self, key: &Value, row_idx: usize) {
        self.entries
            .retain(|(k, idx)| !(k == key && *idx == row_idx));
    }

    fn lookup(&self, key: &Value) -> Option<&[usize]> {
        let mut results = Vec::new();
        for (k, idx) in &self.entries {
            if k == key {
                results.push(*idx);
            }
        }
        if results.is_empty() {
            None
        } else {
            let slice: &'static [usize] = Box::leak(results.into_boxed_slice());
            Some(slice)
        }
    }
}

#[derive(Default)]
struct MockStorage {
    tables: HashMap<String, Vec<Row>>,
}

impl StorageEngine for MockStorage {
    fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        if self.tables.contains_key(schema.name()) {
            return Err(DbError::invalid_operation("table exists"));
        }
        let _ = self.tables.insert(schema.name().to_string(), Vec::new());
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> Result<(), DbError> {
        if self.tables.remove(name).is_none() {
            return Err(DbError::table_not_found(name));
        }
        Ok(())
    }

    fn rename_table(&mut self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        if let Some(rows) = self.tables.remove(old_name) {
            let _ = self.tables.insert(new_name.to_string(), rows);
            Ok(())
        } else {
            Err(DbError::table_not_found(old_name))
        }
    }

    fn insert_row(&mut self, table: &str, row: &Row) -> Result<(), DbError> {
        let rows = self
            .tables
            .get_mut(table)
            .ok_or_else(|| DbError::table_not_found(table))?;
        rows.push(row.clone());
        Ok(())
    }

    fn get_row(&mut self, table: &str, row_idx: usize) -> Result<Option<Row>, DbError> {
        let rows = self
            .tables
            .get(table)
            .ok_or_else(|| DbError::table_not_found(table))?;
        Ok(rows.get(row_idx).cloned())
    }

    fn set_cell(
        &mut self,
        table: &str,
        row_idx: usize,
        col_idx: usize,
        value: Value,
    ) -> Result<(), DbError> {
        let rows = self
            .tables
            .get_mut(table)
            .ok_or_else(|| DbError::table_not_found(table))?;
        let row = rows
            .get_mut(row_idx)
            .ok_or_else(|| DbError::invalid_operation("row out of bounds"))?;
        if col_idx >= row.len() {
            return Err(DbError::invalid_operation("column out of bounds"));
        }
        let mut new_values = row.values().to_vec();
        if let Some(cell) = new_values.get_mut(col_idx) {
            *cell = value;
        } else {
            return Err(DbError::invalid_operation("column out of bounds"));
        }
        *row = Row::new(new_values);
        Ok(())
    }

    fn replace_rows(&mut self, table: &str, rows: Vec<Row>) -> Result<(), DbError> {
        let existing = self
            .tables
            .get_mut(table)
            .ok_or_else(|| DbError::table_not_found(table))?;
        *existing = rows;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn get_row_by_key(&mut self, _table: &str, _key: &Value) -> Result<Option<Row>, DbError> {
        Err(DbError::unsupported(
            "get_row_by_key not implemented in InMemoryEngine",
        ))
    }
}

#[derive(Default)]
struct MockTableStore {
    rows: Vec<Row>,
}

impl TableStore for MockTableStore {
    fn insert(&mut self, row: &Row) -> Result<(), DbError> {
        self.rows.push(row.clone());
        Ok(())
    }

    fn set_cell(&mut self, row_idx: usize, col_idx: usize, value: Value) -> Result<(), DbError> {
        let row = self
            .rows
            .get_mut(row_idx)
            .ok_or_else(|| DbError::invalid_operation("row out of bounds"))?;
        if col_idx >= row.len() {
            return Err(DbError::invalid_operation("col out of bounds"));
        }
        let mut values = row.values().to_vec();
        if let Some(cell) = values.get_mut(col_idx) {
            *cell = value;
        } else {
            return Err(DbError::invalid_operation("col out of bounds"));
        }
        *row = Row::new(values);
        Ok(())
    }

    fn replace_rows(&mut self, rows: Vec<Row>) -> Result<(), DbError> {
        self.rows = rows;
        Ok(())
    }
}

#[test]
fn test_catalog_store() {
    let mut cat = MockCatalog::default();
    let schema_result = TableSchema::try_new("t1", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(schema_result.is_ok());
    if let Ok(schema) = schema_result {
        assert!(cat.create_table(schema).is_ok());
    }
    assert!(cat.tables.contains_key("t1"));
    let schema_result_2 = TableSchema::try_new("t1", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(schema_result_2.is_ok());
    if let Ok(schema_2) = schema_result_2 {
        assert!(cat.create_table(schema_2).is_err());
    }
    assert!(cat.drop_table("t1").is_ok());
    assert!(!cat.tables.contains_key("t1"));
    assert!(cat.drop_table("t1").is_err());
    let schema_result_3 =
        TableSchema::try_new("old", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(schema_result_3.is_ok());
    if let Ok(schema_3) = schema_result_3 {
        assert!(cat.create_table(schema_3).is_ok());
    }
    assert!(cat.rename_table("old", "new").is_ok());
    assert!(cat.tables.contains_key("new"));
}

#[test]
fn test_index_trait() {
    let mut idx = MockIndex::default();
    let key1 = Value::from(1i64);
    let key2 = Value::from(2i64);
    idx.insert(&key1, 10);
    idx.insert(&key2, 20);
    idx.insert(&key1, 30);
    if let Some(result) = idx.lookup(&key1) {
        assert_eq!(result.len(), 2);
        assert!(result.contains(&10));
        assert!(result.contains(&30));
    } else {
        unreachable!("lookup failed");
    }
    idx.remove(&key1, 10);
    if let Some(result) = idx.lookup(&key1) {
        assert_eq!(result.len(), 1);
        assert_eq!(result.first().copied(), Some(30));
    } else {
        unreachable!("lookup failed after remove");
    }
    assert!(idx.lookup(&key2).is_some());
    assert!(idx.lookup(&Value::Null).is_none());
}

#[test]
fn test_storage_engine_trait() {
    let mut storage = MockStorage::default();
    let schema_result =
        TableSchema::try_new("users", vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(schema_result.is_ok());
    if let Ok(schema) = schema_result {
        assert!(storage.create_table(schema).is_ok());
    }
    let row = Row::new(vec![Value::from(1i64)]);
    assert!(storage.insert_row("users", &row).is_ok());
    if let Ok(Some(got)) = storage.get_row("users", 0) {
        assert_eq!(got, row);
    } else {
        unreachable!("get_row failed");
    }
    assert!(StorageEngine::set_cell(&mut storage, "users", 0, 0, Value::from(99i64)).is_ok());
    if let Ok(Some(modified)) = storage.get_row("users", 0) {
        if let Some(v) = modified.get(0) {
            assert_eq!(*v, Value::from(99i64));
        } else {
            unreachable!("missing value");
        }
    } else {
        unreachable!("get_row failed");
    }
    let new_rows = vec![
        Row::new(vec![Value::from(5i64)]),
        Row::new(vec![Value::from(6i64)]),
    ];
    assert!(StorageEngine::replace_rows(&mut storage, "users", new_rows).is_ok());
    if let Some(rows) = storage.tables.get("users") {
        assert_eq!(rows.len(), 2);
    } else {
        unreachable!("table not found");
    }
    assert!(storage.checkpoint().is_ok());
}

#[test]
fn test_table_store_trait() {
    let mut ts = MockTableStore::default();
    let row = Row::new(vec![Value::from(1i64), Value::Null]);
    assert!(ts.insert(&row).is_ok());
    assert_eq!(ts.rows.len(), 1);
    let text_result = Text::try_new("hello".to_string());
    assert!(text_result.is_ok());
    if let Ok(text) = text_result {
        assert!(ts.set_cell(0, 1, Value::from(text)).is_ok());
    }
    if let Some(modified) = ts.rows.first() {
        if let Some(v) = modified.get(1) {
            if let Value::Text(t) = v {
                assert_eq!(t.as_str(), "hello");
            } else {
                unreachable!("expected text");
            }
        } else {
            unreachable!("missing value");
        }
    } else {
        unreachable!("missing row");
    }
    let new_rows = vec![Row::new(vec![Value::from(2i64)])];
    assert!(ts.replace_rows(new_rows).is_ok());
    assert_eq!(ts.rows.len(), 1);
}
