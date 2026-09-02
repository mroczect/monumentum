use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{CheckConstraint, ColumnDef, ComparisonOp, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn create_column(name: &str, data_type: DataType, unique: bool, nullable: bool) -> ColumnDef {
    let mut col = ColumnDef::new(name, data_type);
    col.set_unique(unique);
    col.set_nullable(nullable);
    col
}

fn create_schema(columns: Vec<ColumnDef>) -> Result<TableSchema, DbError> {
    TableSchema::try_new("test_table", columns)
}

fn create_table(columns: Vec<ColumnDef>) -> Result<Table, DbError> {
    Ok(Table::new(create_schema(columns)?))
}

#[test]
fn new_creates_table_with_correct_schema_and_empty_rows() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false);
    let table = create_table(vec![col])?;
    assert_eq!(table.schema().name(), "test_table");
    assert_eq!(table.schema().columns().len(), 1);
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
    assert!(table.get(0).is_none());
    Ok(())
}

#[test]
fn insert_valid_row_success() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::from(1_i64)]);
    table.insert(row)?;
    assert_eq!(table.len(), 1);
    assert!(!table.is_empty());
    assert!(table.get(0).is_some());
    Ok(())
}

#[test]
fn insert_wrong_number_of_values_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::from(1_i64), Value::from(2_i64)]);
    let result = table.insert(row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: expected 1 values, got 2");
    }
    Ok(())
}

#[test]
fn insert_with_default_value_applies_default() -> Result<(), DbError> {
    let mut col = create_column("age", DataType::Integer, false, true);
    col.set_default(Some(Value::from(30_i64)));
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::Null]);
    table.insert(row)?;
    if let Some(inserted) = table.get(0) {
        assert_eq!(inserted.get(0), Some(&Value::from(30_i64)));
    } else {
        assert!(table.get(0).is_some());
    }
    Ok(())
}

#[test]
fn insert_type_mismatch_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::from("not integer")]);
    let result = table.insert(row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: column 'id' expects INTEGER, got text"
        );
    }
    Ok(())
}

#[test]
fn insert_null_in_non_nullable_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::Null]);
    let result = table.insert(row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: column 'id' is not nullable"
        );
    }
    Ok(())
}

#[test]
fn insert_check_constraint_violation_returns_error() -> Result<(), DbError> {
    let check = CheckConstraint {
        column: "age".to_string(),
        op: ComparisonOp::Gt,
        value: Value::from(0_i64),
    };
    let mut col = create_column("age", DataType::Integer, false, true);
    col.set_check(Some(check));
    let mut table = create_table(vec![col])?;
    let row = Row::new(vec![Value::from(-5_i64)]);
    let result = table.insert(row);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("check constraint failed"));
    }
    Ok(())
}

#[test]
fn insert_duplicate_unique_value_returns_error() -> Result<(), DbError> {
    let col = create_column("email", DataType::Text, true, false);
    let mut table = create_table(vec![col])?;

    table.insert(Row::new(vec![Value::from("a@b.com")]))?;
    let result = table.insert(Row::new(vec![Value::from("a@b.com")]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: duplicate value for column 'email'"
        );
    }
    Ok(())
}

#[test]
fn insert_duplicate_primary_key_returns_error() -> Result<(), DbError> {
    let mut col = create_column("id", DataType::Integer, false, false);
    col.set_primary_key(true);
    let mut table = create_table(vec![col])?;

    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    let result = table.insert(Row::new(vec![Value::from(1_i64)]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: duplicate value for column 'id'"
        );
    }
    Ok(())
}

#[test]
fn insert_null_unique_value_allowed_multiple_times() -> Result<(), DbError> {
    let col = create_column("email", DataType::Text, true, true);
    let mut table = create_table(vec![col])?;

    table.insert(Row::new(vec![Value::Null]))?;
    table.insert(Row::new(vec![Value::Null]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn get_returns_row_at_index() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;

    let row0 = table.get(0);
    assert!(row0.is_some());
    if let Some(row) = row0 {
        assert_eq!(row.get(0), Some(&Value::from(1_i64)));
    }

    let row1 = table.get(1);
    assert!(row1.is_some());
    if let Some(row) = row1 {
        assert_eq!(row.get(0), Some(&Value::from(2_i64)));
    }

    assert!(table.get(2).is_none());
    Ok(())
}

#[test]
fn replace_rows_success() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;

    let new_rows = vec![
        Row::new(vec![Value::from(10_i64)]),
        Row::new(vec![Value::from(20_i64)]),
    ];
    table.replace_rows(new_rows)?;
    assert_eq!(table.len(), 2);
    if let Some(row) = table.get(0) {
        assert_eq!(row.get(0), Some(&Value::from(10_i64)));
    } else {
        assert!(table.get(0).is_some());
    }
    if let Some(row) = table.get(1) {
        assert_eq!(row.get(0), Some(&Value::from(20_i64)));
    } else {
        assert!(table.get(1).is_some());
    }
    Ok(())
}

#[test]
fn replace_rows_with_wrong_length_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let bad_rows = vec![Row::new(vec![Value::from(1_i64), Value::from(2_i64)])];
    let result = table.replace_rows(bad_rows);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Invalid operation: expected 1 values, got 2");
    }
    Ok(())
}

#[test]
fn replace_rows_with_duplicate_unique_values_returns_error() -> Result<(), DbError> {
    let col = create_column("email", DataType::Text, true, false);
    let mut table = create_table(vec![col])?;
    let rows = vec![
        Row::new(vec![Value::from("dup@example.com")]),
        Row::new(vec![Value::from("dup@example.com")]),
    ];
    let result = table.replace_rows(rows);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: duplicate value for column 'email'"
        );
    }
    Ok(())
}

#[test]
fn replace_rows_with_invalid_values_returns_error() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    let rows = vec![Row::new(vec![Value::from("bad")])];
    let result = table.replace_rows(rows);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: column 'id' expects INTEGER, got text"
        );
    }
    Ok(())
}

#[test]
fn lookup_by_unique_on_indexed_column_returns_row() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;

    let value = Value::from(2_i64);
    let result = table.lookup_by_unique(0, &value);
    assert!(result.is_some());
    if let Some(row) = result {
        assert_eq!(row.get(0), Some(&value));
    }
    Ok(())
}

#[test]
fn lookup_by_unique_on_non_indexed_column_falls_back_to_linear_scan() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;

    let value = Value::from(2_i64);
    let result = table.lookup_by_unique(0, &value);
    assert!(result.is_some());
    if let Some(row) = result {
        assert_eq!(row.get(0), Some(&value));
    }
    Ok(())
}

#[test]
fn lookup_by_unique_non_existent_value_returns_none() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;

    let value = Value::from(99_i64);
    assert!(table.lookup_by_unique(0, &value).is_none());
    Ok(())
}

#[test]
fn lookup_by_unique_out_of_range_col_idx_returns_none() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, true, false);
    let table = create_table(vec![col])?;
    let value = Value::from(1_i64);
    assert!(table.lookup_by_unique(5, &value).is_none());
    Ok(())
}

#[test]
fn rows_returns_slice_of_all_rows() -> Result<(), DbError> {
    let col = create_column("id", DataType::Integer, false, false);
    let mut table = create_table(vec![col])?;
    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;

    let rows = table.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get(0), Some(&Value::from(1_i64)));
    assert_eq!(rows[1].get(0), Some(&Value::from(2_i64)));
    Ok(())
}
