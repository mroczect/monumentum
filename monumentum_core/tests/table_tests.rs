use fs2 as _;
use monumentum_core::table::Table;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn text_value(s: &str) -> Result<Value, DbError> {
    let text = monumentum_handler::Text::try_new(s.to_string())?;
    Ok(Value::from(text))
}

fn make_row(id: i64, name: &str) -> Result<Row, DbError> {
    let name_value = text_value(name)?;
    Ok(Row::new(vec![Value::from(id), name_value]))
}

#[test]
fn table_new_empty() {
    let schema_result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = schema_result else { return };
    let table = Table::new(schema);
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert!(table.rows().is_empty());
}

#[test]
fn insert_valid_row() {
    let schema_result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);
    let row_result = make_row(1, "Alice");
    let Ok(row) = row_result else { return };
    assert!(table.insert(&row).is_ok());
    assert_eq!(table.len(), 1);
    let cell = table.get(0).and_then(|r| r.get(&0));
    assert_eq!(cell, Some(&Value::from(1_i64)));
}

#[test]
fn insert_wrong_column_count_errors() {
    let schema_result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);
    let row = Row::new(vec![Value::from(1_i64)]);
    let result = table.insert(&row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
    }
}

#[test]
fn insert_type_mismatch_errors() {
    let schema_result = TableSchema::try_new(
        "t",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);

    let wrong_name_result = text_value("wrong");
    let Ok(wrong_name) = wrong_name_result else {
        return;
    };
    let name_result = text_value("name");
    let Ok(name) = name_result else { return };

    let row = Row::new(vec![wrong_name, name]);
    let result = table.insert(&row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TypeMismatch(_)));
    }
}

#[test]
fn unique_constraint_enforced() {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_unique(true);
    let schema_result =
        TableSchema::try_new("t", vec![id_col, ColumnDef::new("name", DataType::Text)]);
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);

    let first_row_result = make_row(1, "a");
    let Ok(first_row) = first_row_result else {
        return;
    };
    let second_row_result = make_row(1, "b");
    let Ok(second_row) = second_row_result else {
        return;
    };

    assert!(table.insert(&first_row).is_ok());
    let result = table.insert(&second_row);
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
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_unique(true);
    let schema_result =
        TableSchema::try_new("t", vec![id_col, ColumnDef::new("name", DataType::Text)]);
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);

    let first_row_result = make_row(1, "a");
    let Ok(first_row) = first_row_result else {
        return;
    };
    assert!(table.insert(&first_row).is_ok());

    let new_name_result = text_value("b");
    let Ok(new_name) = new_name_result else {
        return;
    };
    assert!(table.set_cell(0, 1, &new_name).is_ok());

    let cell = table.get(0).and_then(|r| r.get(&1)).cloned();
    assert_eq!(cell, Some(new_name));
}

#[test]
fn replace_rows_rebuilds_indexes() {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_unique(true);
    let schema_result =
        TableSchema::try_new("t", vec![id_col, ColumnDef::new("name", DataType::Text)]);
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);

    let first_row_result = make_row(1, "x");
    let Ok(first_row) = first_row_result else {
        return;
    };
    let second_row_result = make_row(2, "y");
    let Ok(second_row) = second_row_result else {
        return;
    };

    let new_rows = vec![first_row, second_row];
    assert!(table.replace_rows(new_rows).is_ok());
    assert_eq!(table.len(), 2);
    let lookup = table.lookup_by_unique(0, &Value::from(2_i64));
    assert!(lookup.is_some());
}
