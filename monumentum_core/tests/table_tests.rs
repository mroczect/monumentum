#![allow(unused_crate_dependencies)]

use monumentum_core::table::Table;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;

fn make_schema(name: &str) -> TableSchema {
    let result = TableSchema::try_new(name, vec![ColumnDef::new("id", DataType::Integer)]);
    assert!(result.is_ok());
    result.unwrap_or_else(|_| unreachable!())
}

#[test]
fn test_table_new_and_accessors() {
    let schema = make_schema("users");
    let table = Table::new(schema.clone());
    assert_eq!(table.schema(), &schema);
    assert!(!table.is_read_only());
    assert_eq!(table.schema().name(), "users");
}

#[test]
fn test_table_rename_schema() {
    let schema = make_schema("old");
    let mut table = Table::new(schema);
    let rename_result = table.rename_schema("new");
    assert!(rename_result.is_ok());
    assert_eq!(table.schema().name(), "new");
}

#[test]
fn test_table_read_only_flag() {
    let schema = make_schema("users");
    let mut table = Table::new(schema);
    table.set_read_only(true);
    assert!(table.is_read_only());
    table.set_read_only(false);
    assert!(!table.is_read_only());
}
