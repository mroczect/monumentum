use monumentum_core::catalog::Catalog;
use monumentum_core::table::Table;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::error::DbError;

fn make_schema(name: &str) -> TableSchema {
    let result = TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)]);
    let Ok(schema) = result else {
        unreachable!("schema creation should succeed for valid input")
    };
    schema
}

#[test]
fn new_catalog_is_empty() {
    let cat = Catalog::new();
    assert!(cat.is_empty());
    assert_eq!(cat.len(), 0);
    assert!(cat.tables().next().is_none());
}

#[test]
fn create_table_success() {
    let mut cat = Catalog::new();
    let schema = make_schema("test");
    assert!(cat.create_table(schema).is_ok());
    assert!(!cat.is_empty());
    assert_eq!(cat.len(), 1);
    assert!(cat.get_table("test").is_some());
}

#[test]
fn create_table_duplicate_name_fails() {
    let mut cat = Catalog::new();
    let schema1 = make_schema("dup");
    let schema2 = make_schema("dup");
    assert!(cat.create_table(schema1).is_ok());
    let result = cat.create_table(schema2);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::InvalidOperation(_)));
    }
}

#[test]
fn drop_table_existing_removes() {
    let mut cat = Catalog::new();
    let schema = make_schema("dropme");
    assert!(cat.create_table(schema).is_ok());
    assert!(cat.drop_table("dropme").is_ok());
    assert!(cat.get_table("dropme").is_none());
    assert!(cat.is_empty());
}

#[test]
fn drop_table_missing_returns_error() {
    let mut cat = Catalog::new();
    let result = cat.drop_table("ghost");
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, DbError::TableNotFound(_)));
    }
}

#[test]
fn rename_table_success_preserves_data() {
    let mut cat = Catalog::new();
    let schema = make_schema("old");
    assert!(cat.create_table(schema).is_ok());
    assert!(cat.rename_table("old", "new").is_ok());
    assert!(cat.get_table("old").is_none());
    assert!(cat.get_table("new").is_some());
}

#[test]
fn rename_table_same_name_noop() {
    let mut cat = Catalog::new();
    let schema = make_schema("same");
    assert!(cat.create_table(schema).is_ok());
    assert!(cat.rename_table("same", "same").is_ok());
    assert!(cat.get_table("same").is_some());
}

#[test]
fn replace_table_valid() {
    let mut cat = Catalog::new();
    let schema1 = make_schema("replace");
    assert!(cat.create_table(schema1).is_ok());
    let new_table = Table::new(make_schema("replace"));
    assert!(cat.replace_table("replace", new_table).is_ok());
    let table = cat.get_table("replace");
    assert!(table.is_some());
    if let Some(t) = table {
        assert_eq!(t.len(), 0);
    }
}
