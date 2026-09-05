use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::{FunctionRegistry, QueryBuilder, ScalarFunction};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;

#[derive(Debug)]
struct UpperFunction;

impl ScalarFunction for UpperFunction {
    fn name(&self) -> &str {
        "upper"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("missing arg"))?;
        let s = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("expected text"))?;
        Value::try_from(s.to_uppercase()).map_err(Into::into)
    }
}

#[test]
fn test_custom_scalar_function() -> Result<(), DbError> {
    let path = std::env::temp_dir().join("custom_fn_test.db");
    let mut storage = FileStorage::open(&path, 10)?;

    let schema = TableSchema::try_new("users", vec![ColumnDef::new("name", DataType::Text)])?;
    storage.create_table(schema)?;

    storage.insert_row(
        "users",
        &Row::new(vec![Value::try_from("alice".to_string())?]),
    )?;

    let mut registry = FunctionRegistry::new();
    registry.register_scalar(Box::new(UpperFunction));

    let results: Vec<Value> = QueryBuilder::new(&mut storage, "users")
        .project(|row| {
            let name = row
                .get(0)
                .ok_or_else(|| DbError::type_mismatch("missing name"))?
                .clone();
            let upper = registry
                .get_scalar("upper")
                .ok_or_else(|| DbError::unsupported("missing upper"))?;
            upper.call(&[name])
        })?
        .execute()?;

    assert_eq!(results.len(), 1);
    let value = results
        .first()
        .ok_or_else(|| DbError::invalid_operation("missing result"))?;
    assert_eq!(value.as_str(), Some("ALICE"));

    drop(storage);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    Ok(())
}
