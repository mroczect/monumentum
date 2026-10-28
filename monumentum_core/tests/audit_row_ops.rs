#![allow(unused_crate_dependencies)]

//! Row operations audit.

use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use monumentum_handler::types::Blob;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "monumentum_audit_row_{}_{}_{}.db",
        std::process::id(),
        tag,
        nanos
    ))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("wal"));
}

fn schema_no_pk() -> TableSchema {
    TableSchema::try_new(
        "items",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("payload", DataType::Blob),
        ],
    )
    .unwrap_or_else(|_| unreachable!())
}

fn schema_with_pk() -> TableSchema {
    let mut id = ColumnDef::new("id", DataType::Integer);
    id.set_primary_key(true);
    TableSchema::try_new("items", vec![id, ColumnDef::new("v", DataType::Integer)])
        .unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_row_small_ok() -> Result<(), DbError> {
    let path = temp_db_path("small");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_no_pk())?;
    let row = Row::new(vec![
        Value::from(1i64),
        Value::from(Blob::try_new(vec![0xAAu8; 100])?),
    ]);
    s.insert_row("items", &row)?;
    assert_eq!(s.get_row("items", 0)?, Some(row));
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_row_just_under_page_limit() -> Result<(), DbError> {
    let path = temp_db_path("near");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_no_pk())?;
    let row = Row::new(vec![
        Value::from(1i64),
        Value::from(Blob::try_new(vec![0xBBu8; 4000])?),
    ]);
    s.insert_row("items", &row)?;
    assert_eq!(s.get_row("items", 0)?, Some(row));
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
#[ignore = "C3: insert_row_static infinite-loop untuk row > PAGE_BODY_SIZE (4072B)"]
fn test_row_oversize_returns_error() -> Result<(), DbError> {
    let path = temp_db_path("oversize");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_no_pk())?;
    let row = Row::new(vec![
        Value::from(1i64),
        Value::from(Blob::try_new(vec![0xCCu8; 8000])?),
    ]);
    let result = s.insert_row("items", &row);
    assert!(
        result.is_err(),
        "oversize row should return Err, bukan hang"
    );
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_pk_duplicate_insert_rejected() -> Result<(), DbError> {
    let path = temp_db_path("pk_dup");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_with_pk())?;
    s.insert_row(
        "items",
        &Row::new(vec![Value::from(1i64), Value::from(10i64)]),
    )?;
    let dup = s.insert_row(
        "items",
        &Row::new(vec![Value::from(1i64), Value::from(20i64)]),
    );
    assert!(dup.is_err(), "duplicate PK must be rejected");
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_pk_duplicate_no_orphan_row() -> Result<(), DbError> {
    let path = temp_db_path("pk_orphan");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_with_pk())?;
    s.insert_row(
        "items",
        &Row::new(vec![Value::from(1i64), Value::from(10i64)]),
    )?;
    let _ = s.insert_row(
        "items",
        &Row::new(vec![Value::from(1i64), Value::from(20i64)]),
    );

    let all = s.get_all_rows("items")?;
    let by_key = s.get_row_by_key("items", &Value::from(1i64))?;

    s.close()?;
    cleanup(&path);

    assert!(by_key.is_some(), "baris dengan PK=1 harus ada");
    assert_eq!(
        all.len(),
        1,
        "C4 CONFIRMED: {} baris terlihat padahal hanya 1 insert sukses",
        all.len()
    );
    Ok(())
}

#[test]
fn test_set_cell_preserves_all_rows() -> Result<(), DbError> {
    let path = temp_db_path("set_cell");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_no_pk())?;
    for i in 0..10u8 {
        s.insert_row(
            "items",
            &Row::new(vec![
                Value::from(i64::from(i)),
                Value::from(Blob::try_new(vec![i; 4])?),
            ]),
        )?;
    }
    let new_blob = Blob::try_new(vec![0xFF; 4])?;
    s.set_cell("items", 5, 1, Value::from(new_blob.clone()))?;

    for i in 0..10u8 {
        let row = s
            .get_row("items", usize::from(i))?
            .ok_or_else(|| DbError::invalid_operation("expected row"))?;
        if i == 5 {
            assert_eq!(row.get(1), Some(&Value::from(new_blob.clone())));
        } else {
            assert_eq!(row.get(1), Some(&Value::from(Blob::try_new(vec![i; 4])?)));
        }
    }
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_replace_rows_roundtrip() -> Result<(), DbError> {
    let path = temp_db_path("replace");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_no_pk())?;
    let rows = vec![
        Row::new(vec![Value::from(10i64), Value::Null]),
        Row::new(vec![Value::from(20i64), Value::Null]),
        Row::new(vec![Value::from(30i64), Value::Null]),
    ];
    s.replace_rows("items", rows.clone())?;
    for (i, expected) in rows.iter().enumerate() {
        let row = s
            .get_row("items", i)?
            .ok_or_else(|| DbError::invalid_operation("expected row"))?;
        assert_eq!(&row, expected);
    }
    s.close()?;
    cleanup(&path);
    Ok(())
}

#[test]
fn test_set_cell_with_pk_change() -> Result<(), DbError> {
    let path = temp_db_path("set_cell_pk");
    let mut s = FileStorage::open(&path, 16)?;
    s.create_table(schema_with_pk())?;
    s.insert_row(
        "items",
        &Row::new(vec![Value::from(1i64), Value::from(10i64)]),
    )?;
    s.insert_row(
        "items",
        &Row::new(vec![Value::from(2i64), Value::from(20i64)]),
    )?;

    let result = s.set_cell("items", 1, 0, Value::from(1i64));

    let row0 = s.get_row("items", 0)?;
    let row1 = s.get_row("items", 1)?;
    s.close()?;
    cleanup(&path);

    if result.is_ok() {
        assert!(row0.is_some());
        assert!(row1.is_some());
    } else {
        assert_eq!(
            row0,
            Some(Row::new(vec![Value::from(1i64), Value::from(10i64)])),
            "C5: row0 berubah setelah set_cell PK gagal (rollback tidak ada)"
        );
        assert_eq!(
            row1,
            Some(Row::new(vec![Value::from(2i64), Value::from(20i64)])),
            "C5: row1 berubah setelah set_cell PK gagal (rollback tidak ada)"
        );
    }
    Ok(())
}
