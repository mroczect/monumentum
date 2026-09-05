use core::time::Duration;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile as _;

use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::QueryBuilder;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial_lock() -> Result<MutexGuard<'static, ()>, DbError> {
    SERIAL
        .lock()
        .map_err(|e| DbError::invalid_operation(format!("test mutex poisoned: {e}")))
}

fn setup_storage(label: &str) -> Result<(FileStorage, std::path::PathBuf), DbError> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "query_test_{label}_{}_{nanos}.db",
        std::process::id()
    ));

    // Pastikan tidak ada file atau lock tersisa dari run sebelumnya
    let wal_path = path.with_extension("wal");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(wal_path);

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

    Ok((storage, path))
}

#[test]
fn test_filter_sort_limit() -> Result<(), DbError> {
    let _guard = serial_lock()?;
    let (mut storage, path) = setup_storage("filter_sort_limit")?;

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

    drop(storage);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    Ok(())
}

#[test]
fn test_project() -> Result<(), DbError> {
    let _guard = serial_lock()?;
    let (mut storage, path) = setup_storage("project")?;

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

    drop(storage);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("wal"));
    Ok(())
}
