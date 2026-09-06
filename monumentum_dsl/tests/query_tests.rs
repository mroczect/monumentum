use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::QueryBuilder;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use tempfile::tempdir;

fn setup_storage(label: &str) -> Result<(FileStorage, tempfile::TempDir), DbError> {
    let dir = tempdir().map_err(DbError::from_io)?;
    let path = dir.path().join(format!("{label}.db"));

    let mut storage = FileStorage::open(&path, 10)?;
    let schema = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
            ColumnDef::new("age", DataType::Integer),
        ],
    )?;
    storage.create_table(schema)?;

    for i in 0_i64..10_i64 {
        let row = Row::new(vec![
            Value::from(i),
            Value::try_from(format!("User{i}"))?,
            Value::from(i.saturating_add(20)),
        ]);
        storage.insert_row("users", &row)?;
    }

    Ok((storage, dir))
}

#[test]
fn test_filter_sort_limit() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_storage("filter_sort_limit")?;

    let rows = QueryBuilder::new(&mut storage, "users")
        .filter(|row| {
            let age = row
                .get(2)
                .ok_or_else(|| DbError::type_mismatch("missing age"))?
                .as_i64()
                .ok_or_else(|| DbError::type_mismatch("expected integer"))?;
            Ok(age >= 25)
        })
        .sort_by(|a, b| {
            let age_a = a.get(2).and_then(Value::as_i64).unwrap_or(0);
            let age_b = b.get(2).and_then(Value::as_i64).unwrap_or(0);
            age_b.cmp(&age_a)
        })
        .limit(3)
        .execute()?;

    assert_eq!(rows.len(), 3);
    let first = rows
        .first()
        .ok_or_else(|| DbError::invalid_operation("expected at least one row"))?;
    assert_eq!(first.get(2).and_then(Value::as_i64), Some(29));

    Ok(())
}

#[test]
fn test_project() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_storage("project")?;

    let names = QueryBuilder::new(&mut storage, "users")
        .filter(|row| {
            let age = row
                .get(2)
                .ok_or_else(|| DbError::type_mismatch("missing age"))?
                .as_i64()
                .ok_or_else(|| DbError::type_mismatch("expected integer"))?;
            Ok(age >= 25)
        })
        .project(|row| {
            let name = row
                .get(1)
                .ok_or_else(|| DbError::type_mismatch("missing name"))?
                .clone();
            Ok(name)
        })?
        .limit(5)
        .execute()?;

    assert_eq!(names.len(), 5);

    Ok(())
}
