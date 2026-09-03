use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::store::storage::{FileStorage, InMemoryStorage, StorageEngine};
use std::fs;

use crate::common::TempPath;

fn create_sample_schema() -> Result<TableSchema, DbError> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_primary_key(true);
    TableSchema::try_new("users", vec![id_col])
}

fn create_sample_catalog() -> Result<Catalog, DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_sample_schema()?)?;
    Ok(catalog)
}

fn create_sample_catalog_with_row() -> Result<Catalog, DbError> {
    let mut catalog = create_sample_catalog()?;
    if let Some(table) = catalog.get_table_mut("users") {
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
    }
    Ok(catalog)
}

fn close_storage(storage: FileStorage) -> Result<(), DbError> {
    storage.close()
}

#[test]
fn in_memory_storage_default_is_empty() -> Result<(), DbError> {
    let mut storage = InMemoryStorage::new();
    let catalog = storage.load_catalog()?;
    assert!(catalog.is_empty());
    Ok(())
}

#[test]
fn in_memory_storage_save_and_load() -> Result<(), DbError> {
    let mut storage = InMemoryStorage::new();
    let catalog = create_sample_catalog()?;
    storage.save_catalog(&catalog)?;
    let loaded = storage.load_catalog()?;
    assert_eq!(catalog, loaded);
    Ok(())
}

#[test]
fn in_memory_storage_get_table() -> Result<(), DbError> {
    let mut storage = InMemoryStorage::new();
    let catalog = create_sample_catalog()?;
    storage.save_catalog(&catalog)?;
    assert!(storage.get_table("users").is_some());
    Ok(())
}

#[test]
fn in_memory_storage_mutate_via_catalog() -> Result<(), DbError> {
    let mut storage = InMemoryStorage::new();
    let catalog = create_sample_catalog()?;
    storage.save_catalog(&catalog)?;

    let mut loaded = storage.load_catalog()?;
    if let Some(table) = loaded.get_table_mut("users") {
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
    } else {
        return Err(DbError::table_not_found("users"));
    }
    storage.save_catalog(&loaded)?;

    let final_catalog = storage.load_catalog()?;
    if let Some(table) = final_catalog.get_table("users") {
        assert_eq!(table.len(), 1);
    } else {
        return Err(DbError::table_not_found("users"));
    }
    Ok(())
}

#[test]
fn file_storage_open_creates_new_files() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let storage = FileStorage::open(temp.path())?;

    assert!(temp.path().with_extension("wal").exists());

    close_storage(storage)?;
    Ok(())
}

#[test]
fn file_storage_open_loads_empty_catalog_when_no_data() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let mut storage = FileStorage::open(temp.path())?;
    let catalog = storage.load_catalog()?;
    assert!(catalog.is_empty());
    close_storage(storage)?;
    Ok(())
}

#[test]
fn file_storage_save_and_load_catalog() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let mut storage = FileStorage::open(temp.path())?;
    let catalog = create_sample_catalog()?;
    storage.save_catalog(&catalog)?;
    let loaded = storage.load_catalog()?;
    assert_eq!(catalog, loaded);
    close_storage(storage)?;
    Ok(())
}

#[test]
fn file_storage_persists_across_reopen() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let catalog = create_sample_catalog_with_row()?;

    {
        let mut storage = FileStorage::open(temp.path())?;
        storage.save_catalog(&catalog)?;
        close_storage(storage)?;
    }

    let mut storage2 = FileStorage::open(temp.path())?;
    let loaded = storage2.load_catalog()?;
    assert_eq!(catalog, loaded);
    close_storage(storage2)?;
    Ok(())
}

#[test]
fn file_storage_checkpoint_writes_snapshot_and_truncates_wal() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let mut storage = FileStorage::open(temp.path())?;
    let catalog = create_sample_catalog_with_row()?;
    storage.save_catalog(&catalog)?;
    storage.checkpoint()?;

    let wal_path = temp.path().with_extension("wal");
    let wal_metadata = fs::metadata(&wal_path)?;
    assert_eq!(wal_metadata.len(), 0);

    close_storage(storage)?;
    let mut storage2 = FileStorage::open(temp.path())?;
    let loaded = storage2.load_catalog()?;
    assert_eq!(loaded, catalog);
    close_storage(storage2)?;
    Ok(())
}

#[test]
fn file_storage_recovery_prefers_latest_wal_record() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let mut storage = FileStorage::open(temp.path())?;

    let catalog_v1 = create_sample_catalog()?;
    storage.save_catalog(&catalog_v1)?;

    let catalog_v2 = create_sample_catalog_with_row()?;
    storage.save_catalog(&catalog_v2)?;

    storage.checkpoint()?;

    let mut catalog_v3 = catalog_v2.clone();
    if let Some(table) = catalog_v3.get_table_mut("users") {
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
    }
    storage.save_catalog(&catalog_v3)?;

    close_storage(storage)?;

    let mut storage2 = FileStorage::open(temp.path())?;
    let loaded = storage2.load_catalog()?;
    assert_eq!(loaded, catalog_v3);
    close_storage(storage2)?;
    Ok(())
}

#[test]
fn file_storage_ignores_lower_sequence_wal_record() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let mut storage = FileStorage::open(temp.path())?;

    let catalog_v1 = create_sample_catalog()?;
    storage.save_catalog(&catalog_v1)?;

    let catalog_v2 = create_sample_catalog_with_row()?;
    storage.save_catalog(&catalog_v2)?;

    storage.checkpoint()?;

    close_storage(storage)?;

    let mut storage2 = FileStorage::open(temp.path())?;
    let loaded = storage2.load_catalog()?;
    assert_eq!(loaded, catalog_v2);
    close_storage(storage2)?;
    Ok(())
}

#[test]
fn file_storage_close_unlocks_file() -> Result<(), DbError> {
    let temp = TempPath::new_file("monumentum_storage_test");
    let storage = FileStorage::open(temp.path())?;
    close_storage(storage)?;

    let storage2 = FileStorage::open(temp.path())?;
    close_storage(storage2)?;
    Ok(())
}
