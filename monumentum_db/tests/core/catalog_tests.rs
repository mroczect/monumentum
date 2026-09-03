use monumentum_db::core::catalog::Catalog;
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::{ColumnDef, DataType};
use monumentum_db::core::schema::table_schema::TableSchema;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use proptest::prelude::*;

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

#[test]
fn create_table_with_same_name_different_case_is_allowed() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("Users")?)?;
    catalog.create_table(create_simple_schema("users")?)?;
    assert_eq!(catalog.len(), 2);
    assert!(catalog.get_table("Users").is_some());
    assert!(catalog.get_table("users").is_some());
    Ok(())
}

#[test]
fn drop_table_case_sensitive() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("Users")?)?;
    catalog.create_table(create_simple_schema("users")?)?;
    assert_eq!(catalog.len(), 2);

    catalog.drop_table("Users")?;
    assert_eq!(catalog.len(), 1);
    assert!(catalog.get_table("Users").is_none());
    assert!(catalog.get_table("users").is_some());
    Ok(())
}

#[test]
fn replace_table_success_replaces_content() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;

    let mut new_table = monumentum_db::core::table::Table::new(create_simple_schema("users")?);
    new_table.insert(Row::new(vec![Value::from(42_i64)]))?;
    catalog.replace_table("users", new_table)?;

    if let Some(table) = catalog.get_table("users") {
        assert_eq!(table.len(), 1);
        assert_eq!(
            table.get(0).and_then(|row| row.get(0)),
            Some(&Value::from(42_i64))
        );
    } else {
        return Err(DbError::table_not_found("users"));
    }
    Ok(())
}

#[test]
fn replace_table_with_mismatched_schema_name_returns_error() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;

    let wrong_schema = create_simple_schema("other")?;
    let wrong_table = monumentum_db::core::table::Table::new(wrong_schema);
    let result = catalog.replace_table("users", wrong_table);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("schema name does not match"));
    }
    Ok(())
}

#[test]
fn replace_table_non_existent_returns_error() {
    let mut catalog = Catalog::new();
    let table = monumentum_db::core::table::Table::new(create_simple_schema("users").unwrap());
    let result = catalog.replace_table("missing", table);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Table not found: missing");
    }
}

#[test]
fn rename_table_same_name_returns_ok() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    catalog.rename_table("users", "users")?;
    assert!(catalog.get_table("users").is_some());
    Ok(())
}

#[test]
fn rename_table_to_existing_name_returns_error() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    catalog.create_table(create_simple_schema("products")?)?;
    let result = catalog.rename_table("users", "products");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Invalid operation: table 'products' already exists"
        );
    }
    Ok(())
}

#[test]
fn rename_table_non_existent_returns_error() {
    let mut catalog = Catalog::new();
    let result = catalog.rename_table("missing", "new");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.to_string(), "Table not found: missing");
    }
}

#[test]
fn rename_table_preserves_schema_and_data() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;
    if let Some(table) = catalog.get_table_mut("users") {
        table.insert(Row::new(vec![Value::from(1_i64)]))?;
        table.insert(Row::new(vec![Value::from(2_i64)]))?;
    }

    catalog.rename_table("users", "people")?;

    assert!(catalog.get_table("users").is_none());
    if let Some(table) = catalog.get_table("people") {
        assert_eq!(table.schema().name(), "people");
        assert_eq!(table.len(), 2);
    } else {
        return Err(DbError::table_not_found("people"));
    }
    Ok(())
}

#[test]
fn rename_table_updates_lookup() -> Result<(), DbError> {
    let mut catalog = Catalog::new();
    catalog.create_table(create_simple_schema("users")?)?;

    catalog.rename_table("users", "people")?;
    assert!(catalog.get_table("users").is_none());
    assert!(catalog.get_table("people").is_some());

    catalog.drop_table("people")?;
    assert!(catalog.get_table("people").is_none());
    Ok(())
}

#[test]
fn catalog_with_many_tables_operations() -> Result<(), DbError> {
    const TABLE_COUNT: usize = 1000;
    let mut catalog = Catalog::new();

    for i in 0..TABLE_COUNT {
        catalog.create_table(create_simple_schema(&format!("table_{i}"))?)?;
    }
    assert_eq!(catalog.len(), TABLE_COUNT);
    assert!(!catalog.is_empty());

    for i in (0..TABLE_COUNT).step_by(100) {
        assert!(catalog.get_table(&format!("table_{i}")).is_some());
    }

    for i in 0..TABLE_COUNT {
        catalog.drop_table(&format!("table_{i}"))?;
    }
    assert_eq!(catalog.len(), 0);
    assert!(catalog.is_empty());
    Ok(())
}

#[test]
fn catalog_clone_is_independent() -> Result<(), DbError> {
    let mut original = Catalog::new();
    original.create_table(create_simple_schema("users")?)?;

    let mut cloned = original.clone();
    cloned.drop_table("users")?;
    cloned.create_table(create_simple_schema("products")?)?;

    assert!(original.get_table("users").is_some());
    assert!(original.get_table("products").is_none());
    assert!(cloned.get_table("users").is_none());
    assert!(cloned.get_table("products").is_some());
    Ok(())
}

#[test]
fn catalog_partial_eq_reflects_contents() -> Result<(), DbError> {
    let mut catalog1 = Catalog::new();
    catalog1.create_table(create_simple_schema("users")?)?;
    let catalog2 = catalog1.clone();
    assert_eq!(catalog1, catalog2);

    catalog1.drop_table("users")?;
    assert_ne!(catalog1, catalog2);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn random_table_operations_maintain_invariants(
        operations in prop::collection::vec(0_u8..3, 0..50),
    ) {
        let mut catalog = Catalog::new();
        let mut existing: Vec<String> = Vec::new();
        let mut next_id = 0_usize;

        for op in operations {
            match op % 3 {
                0 => {
                    let name = format!("t_{next_id}");
                    next_id += 1;
                    let schema = TableSchema::try_new(name.clone(), vec![ColumnDef::new("id", DataType::Integer)]).unwrap();
                    if catalog.create_table(schema).is_ok() {
                        existing.push(name);
                    }
                },
                1 => {
                    if !existing.is_empty() {
                        let idx = next_id % existing.len();
                        let name = existing.remove(idx);
                        let _ = catalog.drop_table(&name);
                    }
                },
                _ => {
                    if existing.len() >= 2 {
                        let old_idx = next_id % existing.len();
                        let new_name = format!("r_{next_id}");
                        next_id += 1;
                        let old_name = existing[old_idx].clone();
                        if catalog.rename_table(&old_name, &new_name).is_ok() {
                            existing[old_idx] = new_name;
                        }
                    }
                },
            }

            prop_assert_eq!(catalog.len(), existing.len());
            let table_count = catalog.tables().count();
            prop_assert_eq!(catalog.len(), table_count);
        }
    }
}
