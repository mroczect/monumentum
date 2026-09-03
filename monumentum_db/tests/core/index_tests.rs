use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

#[test]
fn unique_index_on_text_column() -> Result<(), DbError> {
    let mut col = ColumnDef::new("code", DataType::Text);
    col.set_unique(true);
    col.set_nullable(false);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::from("abc")]))?;
    let result = table.insert(Row::new(vec![Value::from("abc")]));
    assert!(result.is_err());
    table.insert(Row::new(vec![Value::from("abd")]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn unique_index_on_float_column() -> Result<(), DbError> {
    let mut col = ColumnDef::new("val", DataType::Float);
    col.set_unique(true);
    col.set_nullable(false);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::try_from(1.5_f64)?]))?;
    let duplicate = Value::try_from(1.5_f64)?;
    let result = table.insert(Row::new(vec![duplicate]));
    assert!(result.is_err());
    table.insert(Row::new(vec![Value::try_from(2.5_f64)?]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn unique_index_on_blob_column() -> Result<(), DbError> {
    let mut col = ColumnDef::new("data", DataType::Blob);
    col.set_unique(true);
    col.set_nullable(false);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::from(vec![1_u8, 2, 3])]))?;
    let result = table.insert(Row::new(vec![Value::from(vec![1_u8, 2, 3])]));
    assert!(result.is_err());
    table.insert(Row::new(vec![Value::from(vec![1_u8, 2, 4])]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn unique_index_on_formula_column() -> Result<(), DbError> {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_unique(true);
    col.set_nullable(false);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::Formula("1+1".to_string())]))?;
    let result = table.insert(Row::new(vec![Value::Formula("1+1".to_string())]));
    assert!(result.is_err());
    table.insert(Row::new(vec![Value::Formula("2+2".to_string())]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn replace_rows_rebuilds_unique_index() -> Result<(), DbError> {
    let col = unique_int_column_for_test();
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;
    let new_rows = vec![
        Row::new(vec![Value::from(3_i64)]),
        Row::new(vec![Value::from(4_i64)]),
    ];
    table.replace_rows(new_rows)?;

    assert!(table.lookup_by_unique(0, &Value::from(1_i64)).is_none());
    assert!(table.lookup_by_unique(0, &Value::from(2_i64)).is_none());
    assert!(table.lookup_by_unique(0, &Value::from(3_i64)).is_some());
    assert!(table.lookup_by_unique(0, &Value::from(4_i64)).is_some());
    Ok(())
}

#[test]
fn set_cell_updates_unique_index() -> Result<(), DbError> {
    let col = unique_int_column_for_test();
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    table.insert(Row::new(vec![Value::from(1_i64)]))?;
    table.insert(Row::new(vec![Value::from(2_i64)]))?;

    table.set_cell(0, 0, Value::from(3_i64))?;

    assert!(table.lookup_by_unique(0, &Value::from(1_i64)).is_none());
    assert!(table.lookup_by_unique(0, &Value::from(3_i64)).is_some());
    assert!(table.lookup_by_unique(0, &Value::from(2_i64)).is_some());

    let result = table.set_cell(1, 0, Value::from(3_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("duplicate value"));
    }
    Ok(())
}

#[test]
fn lookup_by_unique_falls_back_to_linear_scan_for_non_indexed_column() -> Result<(), DbError> {
    let col1 = ColumnDef::new("id", DataType::Integer);
    let mut col2 = ColumnDef::new("name", DataType::Text);
    col2.set_unique(true);
    let schema = TableSchema::try_new("t", vec![col1, col2])?;
    let mut table = Table::new(schema);

    table.insert(Row::new(vec![Value::from(1_i64), Value::from("a")]))?;
    table.insert(Row::new(vec![Value::from(2_i64), Value::from("b")]))?;

    let result = table.lookup_by_unique(0, &Value::from(2_i64));
    assert!(result.is_some());
    if let Some(row) = result {
        assert_eq!(row.get(1), Some(&Value::from("b")));
    }
    Ok(())
}

#[test]
fn unique_index_with_many_null_values() -> Result<(), DbError> {
    let mut col = ColumnDef::new("nullable_id", DataType::Integer);
    col.set_unique(true);
    col.set_nullable(true);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    for _ in 0..10 {
        table.insert(Row::new(vec![Value::Null]))?;
    }
    assert_eq!(table.len(), 10);
    Ok(())
}

#[test]
fn unique_index_duplicate_integer_after_many_inserts() -> Result<(), DbError> {
    let col = unique_int_column_for_test();
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);

    for i in 0..100 {
        table.insert(Row::new(vec![Value::from(i as i64)]))?;
    }
    let result = table.insert(Row::new(vec![Value::from(50_i64)]));
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("duplicate value"));
    }
    Ok(())
}

fn unique_int_column_for_test() -> ColumnDef {
    let mut col = ColumnDef::new("id", DataType::Integer);
    col.set_unique(true);
    col.set_nullable(false);
    col
}
