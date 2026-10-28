#![allow(unused_crate_dependencies)]

//! Catalog growth audit — memicu bug saat katalog > 1 halaman.

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
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
        "monumentum_audit_catalog_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("wal"));
}

fn make_schema(name: &str) -> TableSchema {
    TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)])
        .unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_catalog_10_tables_single_page() -> Result<(), DbError> {
    let path = temp_db_path("single");
    let mut storage = FileStorage::open(&path, 16)?;
    for i in 0..10 {
        storage.create_table(make_schema(&format!("tbl_{i:02}")))?;
    }
    for i in 0..10 {
        assert!(storage.get_table(&format!("tbl_{i:02}")).is_some());
    }
    storage.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_catalog_growth_100_tables() {
    let path = temp_db_path("growth100");
    let mut storage = FileStorage::open(&path, 32).unwrap_or_else(|_| unreachable!());

    let mut failed_at: Option<(usize, String)> = None;
    for i in 0..100 {
        let name = format!("tbl_{i:03}");
        if let Err(e) = storage.create_table(make_schema(&name)) {
            failed_at = Some((i, format!("{e}")));
            break;
        }
    }

    drop(storage);
    cleanup(&path);

    assert!(failed_at.is_none(), "C2 CONFIRMED: {failed_at:?}");
}

#[test]
fn test_catalog_survives_reopen() -> Result<(), DbError> {
    let path = temp_db_path("reopen");
    {
        let mut s = FileStorage::open(&path, 16)?;
        for i in 0..8 {
            s.create_table(make_schema(&format!("t_{i}")))?;
        }
        s.close()?;
    }
    {
        let s = FileStorage::open(&path, 16)?;
        for i in 0..8 {
            assert!(s.get_table(&format!("t_{i}")).is_some(), "t_{i} hilang");
        }
        drop(s);
    }
    cleanup(&path);
    Ok(())
}

#[test]
fn test_catalog_drop_recreate_cycles() -> Result<(), DbError> {
    let path = temp_db_path("cycles");
    let mut s = FileStorage::open(&path, 16)?;
    for round in 0..10 {
        for i in 0..5 {
            s.create_table(make_schema(&format!("r{round}_t{i}")))?;
        }
        for i in 0..5 {
            s.drop_table(&format!("r{round}_t{i}"))?;
        }
    }
    s.close()?;
    cleanup(&path);
    Ok(())
}
