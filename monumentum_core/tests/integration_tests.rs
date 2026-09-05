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
