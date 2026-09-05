use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use monumentum_handler::types::Text;
use proptest as _;
use std::collections::HashMap;

struct InMemoryEngine {
    tables: HashMap<String, Vec<Row>>,
}

impl InMemoryEngine {
    fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
}

impl StorageEngine for InMemoryEngine {
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
        let mut values = row.values().to_vec();
        if let Some(cell) = values.get_mut(col_idx) {
            *cell = value;
        } else {
            return Err(DbError::invalid_operation("column out of bounds"));
        }
        *row = Row::new(values);
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

#[test]
fn test_full_workflow() {
    let mut engine = InMemoryEngine::new();
    let schema_result = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(schema_result.is_ok());
    if let Ok(schema) = schema_result {
        assert!(engine.create_table(schema).is_ok());
    }

    let alice_result = Text::try_new("Alice".to_string());
    assert!(alice_result.is_ok());
    if let Ok(alice) = alice_result {
        let row1 = Row::new(vec![Value::from(1i64), Value::from(alice)]);
        assert!(engine.insert_row("users", &row1).is_ok());
    }

    let bob_result = Text::try_new("Bob".to_string());
    assert!(bob_result.is_ok());
    if let Ok(bob) = bob_result {
        let row2 = Row::new(vec![Value::from(2i64), Value::from(bob)]);
        assert!(engine.insert_row("users", &row2).is_ok());
    }

    if let Ok(Some(got)) = engine.get_row("users", 0) {
        assert_eq!(got.values().len(), 2);
        assert_eq!(got.get(0), Some(&Value::from(1i64)));
        if let Some(name) = got.get(1) {
            if let Value::Text(t) = name {
                assert_eq!(t.as_str(), "Alice");
            } else {
                unreachable!("expected text");
            }
        }
    } else {
        unreachable!("get_row failed");
    }

    let bobby_result = Text::try_new("Bobby".to_string());
    assert!(bobby_result.is_ok());
    if let Ok(bobby) = bobby_result {
        assert!(engine.set_cell("users", 1, 1, Value::from(bobby)).is_ok());
    }

    if let Ok(Some(updated)) = engine.get_row("users", 1) {
        if let Some(name) = updated.get(1) {
            if let Value::Text(t) = name {
                assert_eq!(t.as_str(), "Bobby");
            } else {
                unreachable!("expected text");
            }
        }
    } else {
        unreachable!("get_row failed");
    }

    let new_rows = vec![Row::new(vec![Value::from(3i64), Value::Null])];
    assert!(engine.replace_rows("users", new_rows).is_ok());
    if let Ok(Some(row0)) = engine.get_row("users", 0) {
        assert_eq!(row0.values().len(), 2);
        assert_eq!(row0.get(0), Some(&Value::from(3i64)));
        assert_eq!(row0.get(1), Some(&Value::Null));
    } else {
        unreachable!("get_row failed");
    }
    assert!(matches!(engine.get_row("users", 1), Ok(None)));

    assert!(engine.drop_table("users").is_ok());
    assert!(engine.get_row("users", 0).is_err());
}
