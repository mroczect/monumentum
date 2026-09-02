use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::table::Table;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn unique_int_column(name: &str) -> ColumnDef {
    let mut col = ColumnDef::new(name, DataType::Integer);
    col.set_unique(true);
    col.set_nullable(false);
    col
}

#[test]
fn index_prevents_duplicate_on_unique_column() -> Result<(), DbError> {
    let col = unique_int_column("id");
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);
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
fn index_allows_null_multiple_times_on_unique_column() -> Result<(), DbError> {
    let mut col = unique_int_column("id");
    col.set_nullable(true);
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);
    table.insert(Row::new(vec![Value::Null]))?;
    table.insert(Row::new(vec![Value::Null]))?;
    assert_eq!(table.len(), 2);
    Ok(())
}

#[test]
fn index_supports_lookup_by_unique() -> Result<(), DbError> {
    let col = unique_int_column("id");
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);
    table.insert(Row::new(vec![Value::from(10_i64)]))?;
    table.insert(Row::new(vec![Value::from(20_i64)]))?;

    let value = Value::from(20_i64);
    let found = table.lookup_by_unique(0, &value);
    assert!(found.is_some());
    if let Some(row) = found {
        assert_eq!(row.get(0), Some(&value));
    }
    Ok(())
}

#[test]
fn index_is_cleared_after_replace_rows() -> Result<(), DbError> {
    let col = unique_int_column("id");
    let mut table = Table::new(TableSchema::try_new("t", vec![col])?);
    table.insert(Row::new(vec![Value::from(1_i64)]))?;

    let new_rows = vec![Row::new(vec![Value::from(2_i64)])];
    table.replace_rows(new_rows)?;

    let old_value = Value::from(1_i64);
    assert!(table.lookup_by_unique(0, &old_value).is_none());

    let new_value = Value::from(2_i64);
    assert!(table.lookup_by_unique(0, &new_value).is_some());
    Ok(())
}
