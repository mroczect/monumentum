use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::QueryBuilder;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;

#[test]
fn test_sql_injection_attempt_is_literal() -> Result<(), DbError> {
    let path = std::env::temp_dir().join("injection_test.db");
    let mut storage = FileStorage::open(&path, 10)?;

    let schema = TableSchema::try_new("users", vec![ColumnDef::new("name", DataType::Text)])?;
    storage.create_table(schema)?;

    storage.insert_row(
        "users",
        &Row::new(vec![Value::try_from("Alice".to_string())?]),
    )?;
    storage.insert_row(
        "users",
        &Row::new(vec![Value::try_from("' OR '1'='1".to_string())?]),
    )?;

    let results = QueryBuilder::new(&mut storage, "users")
        .filter(|row| {
            let name = row
                .get(0)
                .ok_or_else(|| DbError::type_mismatch("missing name"))?
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("expected text"))?;
            Ok(name == "Alice")
        })
        .execute()?;

    assert_eq!(results.len(), 1);
    assert_eq!(
        results
            .first()
            .and_then(|r| r.get(0))
            .and_then(Value::as_str),
        Some("Alice")
    );

    drop(storage);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    Ok(())
}
