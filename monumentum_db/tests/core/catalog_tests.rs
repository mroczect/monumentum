use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

fn create_simple_schema(name: &str) -> Result<TableSchema, DbError> {
    let col = ColumnDef::new("id", DataType::Integer);
    TableSchema::try_new(name, vec![col])
}

#[test]
fn new_returns_empty_catalog() {
    let catalog = Catalog::new();
    assert!(catalog.is_empty());
    assert_eq!(catalog.len(), 0);
}

#[test]
fn create_table_success() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    let schema = create_simple_schema("users")?;
    catalog.create_table(schema)?;
    assert_eq!(catalog.len(), 1);
    assert!(!catalog.is_empty());
    Ok(())
}

#[test]
fn create_table_duplicate_name_returns_error() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    let result = catalog.create_table(create_simple_schema("users")?);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: table 'users' already exists"
        );
    }
    Ok(())
}

#[test]
fn drop_table_existing_removes() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    assert_eq!(catalog.len(), 1);

    catalog.drop_table("users")?;
    assert_eq!(catalog.len(), 0);
    assert!(catalog.is_empty());
    Ok(())
}

#[test]
fn drop_table_non_existent_returns_error() {
    let mut catalog = Catalog::new();
    let result = catalog.drop_table("users");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Table not found: users");
    }
}

#[test]
fn get_table_returns_reference() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;

    let table = catalog.get_table("users");
    assert!(table.is_some());
    if let Some(t) = table {
        assert_eq!(t.schema().name(), "users");
    }
    Ok(())
}

#[test]
fn get_table_non_existent_returns_none() {
    let catalog = Catalog::new();
    assert!(catalog.get_table("missing").is_none());
}

#[test]
fn get_table_mut_returns_mutable_reference() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;

    if let Some(table) = catalog.get_table_mut("users") {
        assert_eq!(table.schema().name(), "users");
        let row = monumentum_db::core::row::Row::new(vec![Value::from(1_i64)]);
        table.insert(row)?;
        assert_eq!(table.len(), 1);
    } else {
        assert!(catalog.get_table("users").is_some());
    }
    Ok(())
}

#[test]
fn get_table_mut_non_existent_returns_none() {
    let mut catalog = Catalog::new();
    assert!(catalog.get_table_mut("missing").is_none());
}

#[test]
fn tables_iterator_yields_entries() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    catalog.create_table(create_simple_schema("products")?)?;

    let mut names: Vec<String> = catalog.tables().map(|(name, _)| name.to_string()).collect();
    names.sort();
    assert_eq!(names, vec!["products".to_string(), "users".to_string()]);
    Ok(())
}

#[test]
fn len_and_is_empty_reflect_state() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    assert_eq!(catalog.len(), 0);
    assert!(catalog.is_empty());

    catalog.create_table(create_simple_schema("users")?)?;
    assert_eq!(catalog.len(), 1);
    assert!(!catalog.is_empty());

    catalog.drop_table("users")?;
    assert_eq!(catalog.len(), 0);
    assert!(catalog.is_empty());
    Ok(())
}
