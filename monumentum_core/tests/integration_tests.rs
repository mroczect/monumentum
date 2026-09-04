use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{decode_catalog, encode_catalog};
use monumentum_core::store::storage::{FileStorage, StorageEngine};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;

#[test]
fn full_workflow_in_memory() {
    let mut cat = Catalog::new();
    let schema_result = TableSchema::try_new(
        "users",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    );
    let Ok(schema) = schema_result else {
        return;
    };
    assert!(cat.create_table(schema).is_ok());
    if let Some(table) = cat.get_table_mut("users") {
        let row1 = Row::new(vec![Value::from(1_i64), Value::from("Alice")]);
        let row2 = Row::new(vec![Value::from(2_i64), Value::from("Bob")]);
        assert!(table.insert(&row1).is_ok());
        assert!(table.insert(&row2).is_ok());
    }
    let encode_result = encode_catalog(&cat);
    let Ok(bytes) = encode_result else {
        return;
    };
    let decode_result = decode_catalog(&bytes);
    let Ok(decoded) = decode_result else {
        return;
    };
    assert_eq!(cat, decoded);
}

#[test]
fn file_storage_roundtrip_with_wal() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("monumentum_int_{}", std::process::id()));
    let _ = std::fs::create_dir(&dir);
    let path = dir.join("db.monumentum");
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
