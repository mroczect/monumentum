use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::{AvgFunction, CountFunction, QueryBuilder, SumFunction};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use tempfile::tempdir;

fn setup() -> Result<(FileStorage, tempfile::TempDir), DbError> {
    let dir = tempdir().map_err(DbError::from_io)?;
    let path = dir.path().join("agg_test.db");
    let mut storage = FileStorage::open(&path, 10)?;
    let schema = TableSchema::try_new("numbers", vec![ColumnDef::new("n", DataType::Integer)])?;
    storage.create_table(schema)?;
    for i in 1_i64..=5_i64 {
        storage.insert_row("numbers", &Row::new(vec![Value::from(i)]))?;
    }
    Ok((storage, dir))
}

#[test]
fn test_aggregate_sum() -> Result<(), DbError> {
    let (mut storage, _dir) = setup()?;
    let sum = QueryBuilder::new(&mut storage, "numbers").aggregate(&SumFunction, |row| {
        let value = row
            .get(0)
            .ok_or_else(|| DbError::type_mismatch("missing column"))?
            .clone();
        Ok(value)
    })?;
    assert_eq!(sum.as_i64(), Some(15));
    Ok(())
}

#[test]
fn test_aggregate_avg() -> Result<(), DbError> {
    let (mut storage, _dir) = setup()?;
    let avg = QueryBuilder::new(&mut storage, "numbers").aggregate(&AvgFunction, |row| {
        let value = row
            .get(0)
            .ok_or_else(|| DbError::type_mismatch("missing column"))?
            .clone();
        Ok(value)
    })?;
    if let Value::Float(f) = avg {
        assert!((f.as_f64() - 3.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_aggregate_count() -> Result<(), DbError> {
    let (mut storage, _dir) = setup()?;
    let count = QueryBuilder::new(&mut storage, "numbers")
        .aggregate(&CountFunction, |_row| Ok(Value::from(1_i64)))?;
    assert_eq!(count.as_i64(), Some(5));
    Ok(())
}

#[test]
fn test_aggregate_by_name() -> Result<(), DbError> {
    let (mut storage, _dir) = setup()?;
    let sum = QueryBuilder::new(&mut storage, "numbers").aggregate_by_name("sum", |row| {
        let value = row
            .get(0)
            .ok_or_else(|| DbError::type_mismatch("missing column"))?
            .clone();
        Ok(value)
    })?;
    assert_eq!(sum.as_i64(), Some(15));
    Ok(())
}
