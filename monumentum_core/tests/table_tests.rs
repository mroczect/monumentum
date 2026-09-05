use fs2 as _;
use monumentum_core::table::Table;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;

#[test]
fn table_metadata_basics() {
    let schema_result = TableSchema::try_new("t", vec![ColumnDef::new("id", DataType::Integer)]);
    let Ok(schema) = schema_result else { return };
    let table = Table::new(schema);
    assert_eq!(table.schema().name(), "t");
    assert!(!table.is_read_only());
    let mut table = table;
    table.set_read_only(true);
    assert!(table.is_read_only());
}

#[test]
fn table_rename_schema() {
    let schema_result = TableSchema::try_new("old", vec![ColumnDef::new("id", DataType::Integer)]);
    let Ok(schema) = schema_result else { return };
    let mut table = Table::new(schema);
    assert!(table.rename_schema("new").is_ok());
    assert_eq!(table.schema().name(), "new");
}
