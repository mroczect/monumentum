#![allow(unused_crate_dependencies)]

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::traits::StorageEngine;
use monumentum_handler::types::Text;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_integration_{}_{}.db",
        std::process::id(),
        nanos
    ))
}

fn make_schema_with_pk() -> TableSchema {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    let name_col = ColumnDef::new("name", DataType::Text);
    let result = TableSchema::try_new("users", vec![id_col, name_col]);
    assert!(result.is_ok());
    result.unwrap_or_else(|_| unreachable!())
}

fn make_schema() -> TableSchema {
    let result = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    assert!(result.is_ok());
    result.unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_file_storage_full_workflow() {
    let path = temp_db_path();
    let storage_result = FileStorage::open(&path, 10);
    assert!(storage_result.is_ok());
    if let Ok(mut storage) = storage_result {
        let schema = make_schema();
        let create_result = storage.create_table(schema);
        assert!(create_result.is_ok());

        let row = Row::new(vec![
            Value::from(1i64),
            Value::from(Text::try_new("Alice".to_string()).unwrap_or_else(|_| unreachable!())),
        ]);
        let insert_result = storage.insert_row("users", &row);
        assert!(insert_result.is_ok());

        let get_result = storage.get_row("users", 0);
        assert!(get_result.is_ok());
        if let Ok(Some(retrieved)) = get_result {
            assert_eq!(retrieved, row);
        } else {
            unreachable!("expected row");
        }

        let checkpoint = storage.checkpoint();
        assert!(checkpoint.is_ok());

        let drop_result = storage.drop_table("users");
        assert!(drop_result.is_ok());
    }
    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(&wal_path);
}

#[test]
fn test_file_storage_reopen_persists_data() {
    let path = temp_db_path();
    let schema = make_schema();
    let row = Row::new(vec![
        Value::from(1i64),
        Value::from(Text::try_new("Bob".to_string()).unwrap_or_else(|_| unreachable!())),
    ]);

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let create_result = storage.create_table(schema);
            assert!(create_result.is_ok());
            let insert_result = storage.insert_row("users", &row);
            assert!(insert_result.is_ok());
            let checkpoint = storage.checkpoint();
            assert!(checkpoint.is_ok());
            let close = storage.close();
            assert!(close.is_ok());
        }
    }

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let get_result = storage.get_row("users", 0);
            assert!(get_result.is_ok());
            if let Ok(Some(retrieved)) = get_result {
                assert_eq!(retrieved, row);
            } else {
                unreachable!("expected row after reopen");
            }
        }
    }
    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(&wal_path);
}

#[test]
fn test_crash_recovery_without_checkpoint() {
    let path = temp_db_path();
    let schema = make_schema();

    let text_result = Text::try_new("Alice".to_string());
    assert!(text_result.is_ok());
    let Ok(text) = text_result else {
        unreachable!("text creation failed");
    };

    let row = Row::new(vec![Value::from(1i64), Value::from(text)]);

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let create_result = storage.create_table(schema);
            assert!(create_result.is_ok());
            let insert_result = storage.insert_row("users", &row);
            assert!(insert_result.is_ok());
        }
    }

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let get_result = storage.get_row("users", 0);
            assert!(get_result.is_ok());
            if let Ok(Some(retrieved)) = get_result {
                assert_eq!(retrieved, row);
            } else {
                unreachable!("expected row after crash recovery");
            }
        }
    }

    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(wal_path);
}

#[test]
fn test_primary_key_lookup() {
    let path = temp_db_path();
    let schema = make_schema_with_pk();
    let text_result = Text::try_new("Alice".to_string());
    assert!(text_result.is_ok());
    let Ok(text) = text_result else {
        unreachable!("text creation failed");
    };
    let row = Row::new(vec![Value::from(1i64), Value::from(text)]);

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let create_result = storage.create_table(schema);
            assert!(create_result.is_ok());
            let insert_result = storage.insert_row("users", &row);
            assert!(insert_result.is_ok());
            let get_by_key = storage.get_row_by_key("users", &Value::from(1i64));
            assert!(get_by_key.is_ok());
            if let Ok(Some(retrieved)) = get_by_key {
                assert_eq!(retrieved, row);
            } else {
                unreachable!("expected row by primary key");
            }
        }
    }

    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(&wal_path);
}

#[test]
fn test_crash_recovery_for_update_and_replace() {
    let path = temp_db_path();
    let schema = make_schema();
    let text_result = Text::try_new("Alice".to_string());
    assert!(text_result.is_ok());
    let Ok(alice) = text_result else {
        unreachable!("text creation failed");
    };
    let row1 = Row::new(vec![Value::from(1i64), Value::from(alice)]);

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            assert!(storage.create_table(schema).is_ok());
            assert!(storage.insert_row("users", &row1).is_ok());

            let bobby_result = Text::try_new("Bobby".to_string());
            assert!(bobby_result.is_ok());
            let Ok(bobby) = bobby_result else {
                unreachable!("text creation failed");
            };
            assert!(storage.set_cell("users", 0, 1, Value::from(bobby)).is_ok());

            let new_row1 = Row::new(vec![Value::from(3i64), Value::Null]);
            let new_row2 = Row::new(vec![Value::from(4i64), Value::Null]);
            assert!(
                storage
                    .replace_rows("users", vec![new_row1, new_row2])
                    .is_ok()
            );
        }
    }

    {
        let storage_result = FileStorage::open(&path, 10);
        assert!(storage_result.is_ok());
        if let Ok(mut storage) = storage_result {
            let get0 = storage.get_row("users", 0);
            assert!(get0.is_ok());
            if let Ok(Some(row)) = get0 {
                assert_eq!(row, Row::new(vec![Value::from(3i64), Value::Null]));
            } else {
                unreachable!("expected row 0");
            }

            let get1 = storage.get_row("users", 1);
            assert!(get1.is_ok());
            if let Ok(Some(row)) = get1 {
                assert_eq!(row, Row::new(vec![Value::from(4i64), Value::Null]));
            } else {
                unreachable!("expected row 1");
            }
        }
    }

    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(wal_path);
}

#[test]
fn test_file_locking_prevents_concurrent_writer() {
    let path = temp_db_path();
    let first = FileStorage::open(&path, 10);
    assert!(first.is_ok());

    let second = FileStorage::open(&path, 10);
    assert!(second.is_err());

    if let Ok(storage) = first {
        drop(storage);
    }
    let _ = fs::remove_file(&path);
    let wal_path = path.with_extension("wal");
    let _ = fs::remove_file(wal_path);
}
