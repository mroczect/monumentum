#![allow(unused_crate_dependencies)]

//! Page leak audit.

use monumentum_core::pager::Pager;
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
        "monumentum_audit_leak_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("wal"));
}

fn schema_pk() -> TableSchema {
    let mut id = ColumnDef::new("id", DataType::Integer);
    id.set_primary_key(true);
    TableSchema::try_new("t", vec![id, ColumnDef::new("v", DataType::Integer)])
        .unwrap_or_else(|_| unreachable!())
}

fn page_count_of(path: &Path) -> Result<u32, DbError> {
    let p = Pager::open(path)?;
    let c = p.page_count();
    drop(p);
    Ok(c)
}

#[test]
fn test_set_cell_does_not_leak_index_pages() -> Result<(), DbError> {
    let path = temp_db_path("set_cell");
    {
        let mut s = FileStorage::open(&path, 32)?;
        s.create_table(schema_pk())?;
        for i in 0..5 {
            s.insert_row("t", &Row::new(vec![Value::from(i), Value::from(i)]))?;
        }
        s.close()?;
    }
    let before = page_count_of(&path)?;
    {
        let mut s = FileStorage::open(&path, 32)?;
        for round in 0..20 {
            s.set_cell("t", 0, 1, Value::from(i64::from(round)))?;
        }
        s.close()?;
    }
    let after = page_count_of(&path)?;
    cleanup(&path);
    let growth = after.saturating_sub(before);
    assert!(
        growth < 5,
        "M2 CONFIRMED: page_count tumbuh +{growth} setelah 20 set_cell \
         (before={before}, after={after})"
    );
    Ok(())
}

#[test]
fn test_drop_table_frees_pages() -> Result<(), DbError> {
    let path = temp_db_path("drop");
    {
        let mut s = FileStorage::open(&path, 32)?;
        for i in 0..10 {
            let schema = TableSchema::try_new(
                format!("t{i}"),
                vec![ColumnDef::new("id", DataType::Integer)],
            )
            .unwrap_or_else(|_| unreachable!());
            s.create_table(schema)?;
            for j in 0..10 {
                s.insert_row(&format!("t{i}"), &Row::new(vec![Value::from(i64::from(j))]))?;
            }
        }
        s.close()?;
    }
    let before = page_count_of(&path)?;
    {
        let mut s = FileStorage::open(&path, 32)?;
        for i in 0..10 {
            s.drop_table(&format!("t{i}"))?;
        }
        s.close()?;
    }
    let after = page_count_of(&path)?;
    cleanup(&path);
    let freed = before.saturating_sub(after);
    assert!(
        freed >= before / 2,
        "M2 CONFIRMED: drop_table hanya membebaskan {freed} page dari {before} \
         (after={after})"
    );
    Ok(())
}

#[test]
fn test_replace_rows_growth() -> Result<(), DbError> {
    let path = temp_db_path("replace");
    {
        let mut s = FileStorage::open(&path, 32)?;
        s.create_table(schema_pk())?;
        for i in 0..5 {
            s.insert_row("t", &Row::new(vec![Value::from(i), Value::from(i)]))?;
        }
        s.close()?;
    }
    let before = page_count_of(&path)?;
    {
        let mut s = FileStorage::open(&path, 32)?;
        for round in 0..20 {
            s.replace_rows(
                "t",
                vec![
                    Row::new(vec![Value::from(i64::from(round)), Value::from(0i64)]),
                    Row::new(vec![Value::from(i64::from(round) + 100), Value::from(0i64)]),
                ],
            )?;
        }
        s.close()?;
    }
    let after = page_count_of(&path)?;
    cleanup(&path);
    let growth = after.saturating_sub(before);
    assert!(
        growth < 5,
        "M2 CONFIRMED (replace_rows): page_count tumbuh +{growth} \
         (before={before}, after={after})"
    );
    Ok(())
}
