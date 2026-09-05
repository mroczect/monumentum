#![allow(unused_crate_dependencies)]

use monumentum_core::catalog::Catalog;
use monumentum_core::table::Table;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::traits::CatalogStore;

fn make_schema(name: &str) -> TableSchema {
    let result = TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_ok());
    result.unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_catalog_create_table() {
    let mut catalog = Catalog::new();
    let schema = make_schema("users");
    let result = catalog.create_table(schema.clone());
    assert!(result.is_ok());
    assert_eq!(catalog.len(), 1);
    assert!(!catalog.is_empty());

    let duplicate = catalog.create_table(schema);
    assert!(duplicate.is_err());
}

#[test]
fn test_catalog_drop_table() {
    let mut catalog = Catalog::new();
    let _ = catalog.create_table(make_schema("users"));
    assert_eq!(catalog.len(), 1);

    let drop_ok = catalog.drop_table("users");
    assert!(drop_ok.is_ok());
    assert_eq!(catalog.len(), 0);
    assert!(catalog.is_empty());

    let drop_missing = catalog.drop_table("users");
    assert!(drop_missing.is_err());
}

#[test]
fn test_catalog_rename_table() {
    let mut catalog = Catalog::new();
    let _ = catalog.create_table(make_schema("old"));
    let rename = catalog.rename_table("old", "new");
    assert!(rename.is_ok());
    assert!(catalog.get_table("new").is_some());
    assert!(catalog.get_table("old").is_none());

    let _ = catalog.create_table(make_schema("other"));
    let rename_conflict = catalog.rename_table("other", "new");
    assert!(rename_conflict.is_err());
}

#[test]
fn test_catalog_replace_table() {
    let mut catalog = Catalog::new();
    let _ = catalog.create_table(make_schema("users"));
    let table = Table::new(make_schema("users"));
    let replace = catalog.replace_table("users", table);
    assert!(replace.is_ok());

    let wrong_table = Table::new(make_schema("wrong"));
    let replace_wrong = catalog.replace_table("users", wrong_table);
    assert!(replace_wrong.is_err());

    let missing_table = Table::new(make_schema("missing"));
    let replace_missing = catalog.replace_table("nonexistent", missing_table);
    assert!(replace_missing.is_err());
}

#[test]
fn test_catalog_get_table_and_iterator() {
    let mut catalog = Catalog::new();
    let _ = catalog.create_table(make_schema("a"));
    let _ = catalog.create_table(make_schema("b"));

    assert!(catalog.get_table("a").is_some());
    assert!(catalog.get_table_mut("a").is_some());
    assert!(catalog.get_table("c").is_none());

    let mut count = 0;
    for (name, _) in catalog.tables() {
        assert!(name == "a" || name == "b");
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_catalog_trait_impl() {
    let mut catalog = Catalog::new();
    let schema = make_schema("trait_table");
    let result = CatalogStore::create_table(&mut catalog, schema.clone());
    assert!(result.is_ok());

    let drop_result = CatalogStore::drop_table(&mut catalog, "trait_table");
    assert!(drop_result.is_ok());

    let _ = CatalogStore::create_table(&mut catalog, schema);
    let rename_result = CatalogStore::rename_table(&mut catalog, "trait_table", "renamed");
    assert!(rename_result.is_ok());
}
