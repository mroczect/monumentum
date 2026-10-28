#![allow(unused_crate_dependencies)]

//! Durabilitas, recovery, checkpoint, WAL.

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_audit_dur_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("wal"));
}

fn schema() -> TableSchema {
    TableSchema::try_new(
        "kv",
        vec![
            ColumnDef::new("k", DataType::Integer),
            ColumnDef::new("v", DataType::Integer),
        ],
    )
    .unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_crash_without_close() -> Result<(), DbError> {
    let path = temp_db_path("crash");
    {
        let mut s = FileStorage::open(&path, 16)?;
        s.create_table(schema())?;
        s.insert_row("kv", &Row::new(vec![Value::from(1i64), Value::from(42i64)]))?;
    }
    {
        let mut s = FileStorage::open(&path, 16)?;
        assert_eq!(
            s.get_row("kv", 0)?,
            Some(Row::new(vec![Value::from(1i64), Value::from(42i64)]))
        );
    }
    cleanup(&path);
    Ok(())
}

#[test]
fn test_checkpoint_persists() -> Result<(), DbError> {
    let path = temp_db_path("ckpt");
    {
        let mut s = FileStorage::open(&path, 16)?;
        s.create_table(schema())?;
        for i in 0..20 {
            s.insert_row("kv", &Row::new(vec![Value::from(i), Value::from(i * 10)]))?;
        }
        s.checkpoint()?;
    }
    {
        let mut s = FileStorage::open(&path, 16)?;
        for i in 0..20 {
            let idx = usize::try_from(i).unwrap_or_else(|_| unreachable!());
            assert_eq!(
                s.get_row("kv", idx)?,
                Some(Row::new(vec![Value::from(i), Value::from(i * 10)]))
            );
        }
    }
    cleanup(&path);
    Ok(())
}

#[test]
fn test_multiple_crashes_and_recovery() -> Result<(), DbError> {
    let path = temp_db_path("multi");
    {
        let mut s = FileStorage::open(&path, 16)?;
        s.create_table(schema())?;
    }
    for i in 0..5 {
        let mut s = FileStorage::open(&path, 16)?;
        s.insert_row("kv", &Row::new(vec![Value::from(i), Value::from(i)]))?;
    }
    {
        let mut s = FileStorage::open(&path, 16)?;
        for i in 0..5 {
            let idx = usize::try_from(i).unwrap_or_else(|_| unreachable!());
            assert!(s.get_row("kv", idx)?.is_some(), "row {i} hilang");
        }
    }
    cleanup(&path);
    Ok(())
}

#[test]
fn test_wal_truncated_after_checkpoint() -> Result<(), DbError> {
    let path = temp_db_path("wal_trunc");
    let wal_path = path.with_extension("wal");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema())?;
    for i in 0..10 {
        s.insert_row("kv", &Row::new(vec![Value::from(i), Value::from(i)]))?;
    }
    s.checkpoint()?;
    let meta = fs::metadata(&wal_path)?;
    let len = meta.len();
    drop(s);
    cleanup(&path);
    assert_eq!(len, 0, "WAL tidak kosong setelah checkpoint: {len} byte");
    Ok(())
}
