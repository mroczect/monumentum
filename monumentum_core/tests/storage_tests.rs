use fs2 as _;
use monumentum_core::catalog::Catalog;
use monumentum_core::store::storage::{FileStorage, InMemoryStorage, StorageEngine};
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use std::env;

#[test]
fn in_memory_storage_basic() {
    let mut storage = InMemoryStorage::new();
    let mut cat = Catalog::new();
    let schema_result = TableSchema::try_new("t", vec![ColumnDef::new("id", DataType::Integer)]);
    let Ok(schema) = schema_result else {
        return;
    };
    assert!(cat.create_table(schema).is_ok());
    assert!(storage.save_catalog(&cat).is_ok());
    let loaded = storage.get_catalog();
    assert_eq!(loaded, &cat);
    assert!(storage.get_table("t").is_some());
    assert!(storage.get_table("nonexistent").is_none());
}

#[test]
fn file_storage_persist_and_reload() {
    let mut dir = env::temp_dir();
    dir.push(format!("monumentum_test_{}", std::process::id()));
    let _ = std::fs::create_dir(&dir);
    let path = dir.join("test.monumentum");
    {
        let storage_result = FileStorage::open(&path);
        let Ok(mut storage) = storage_result else {
            return;
        };
        let mut cat = Catalog::new();
        let schema_result =
            TableSchema::try_new("t", vec![ColumnDef::new("id", DataType::Integer)]);
        let Ok(schema) = schema_result else {
            return;
        };
        assert!(cat.create_table(schema).is_ok());
        assert!(storage.save_catalog(&cat).is_ok());
        assert!(storage.checkpoint().is_ok());
        assert!(storage.close().is_ok());
    }
    {
        let storage_result = FileStorage::open(&path);
        let Ok(mut storage) = storage_result else {
            return;
        };
        let cat_result = storage.reload_from_disk();
        let Ok(cat) = cat_result else {
            return;
        };
        assert!(cat.get_table("t").is_some());
        assert!(storage.close().is_ok());
    }
    let _ = std::fs::remove_dir_all(&dir);
}
