use monumentum_core::table::Table;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn simple_schema() -> TableSchema {
    let result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = result else {
        unreachable!("schema creation should succeed for valid input")
    };
    schema
}

fn make_row(id: i64, name: &str) -> Row {
    Row::new(vec![Value::from(id), Value::from(name)])
}

#[test]
fn table_new_empty() {
    let table = Table::new(simple_schema());
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert!(table.rows().is_empty());
}

#[test]
fn insert_valid_row() {
    let mut table = Table::new(simple_schema());
    let row = make_row(1, "Alice");
    let result = table.insert(&row);
    assert!(result.is_ok(), "unexpected error: {:?}", result);
    assert_eq!(table.len(), 1);
    assert_eq!(
        table.get(0).and_then(|r| r.get(&0)),
        Some(&Value::from(1_i64))
    );
}

#[test]
fn insert_wrong_column_count_errors() {
    let mut table = Table::new(simple_schema());
    let row = Row::new(vec![Value::from(1)]);
    let result = table.insert(&row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
    }
}

#[test]
fn insert_type_mismatch_errors() {
    let mut table = Table::new(simple_schema());
    let row = Row::new(vec![Value::from("wrong"), Value::from("name")]);
    let result = table.insert(&row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TypeMismatch(_)));
    }
}

#[test]
fn unique_constraint_enforced() {
    let schema_result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(mut schema) = schema_result else {
        unreachable!("schema creation should succeed")
    };
    if let Some(col) = schema.get_column_mut(0) {
        col.set_unique(true);
    }
    let mut table = Table::new(schema);
    let row1 = Row::new(vec![Value::from(1_i64), Value::from("a")]);
    let row2 = Row::new(vec![Value::from(1_i64), Value::from("b")]);
    assert!(table.insert(&row1).is_ok());
    let result = table.insert(&row2);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            DbError::ConstraintViolation {
                kind: monumentum_handler::error::ErrorKind::UniqueViolation,
                ..
            }
        ));
    }
}

#[test]
fn set_cell_updates_value_and_index() {
    let mut schema = simple_schema();
    if let Some(col) = schema.get_column_mut(0) {
        col.set_unique(true);
    }
    let mut table = Table::new(schema);
    let row = make_row(1, "a");
    let result = table.insert(&row);
    assert!(result.is_ok(), "unexpected error: {:?}", result);
    let result = table.set_cell(0, 1, Value::from("b"));
    assert!(result.is_ok(), "unexpected error: {:?}", result);
    let cell = table.get(0).and_then(|r| r.get(&1)).cloned();
    assert_eq!(cell, Some(Value::from("b")));
}

#[test]
fn replace_rows_rebuilds_indexes() {
    let mut schema = simple_schema();
    if let Some(col) = schema.get_column_mut(0) {
        col.set_unique(true);
    }
    let mut table = Table::new(schema);
    let rows = vec![make_row(1, "x"), make_row(2, "y")];
    let result = table.replace_rows(rows);
    assert!(result.is_ok(), "unexpected error: {:?}", result);
    assert_eq!(table.len(), 2);
    assert!(table.lookup_by_unique(0, &Value::from(2_i64)).is_some());
}
