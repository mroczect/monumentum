use core::error::Error;
use fs2 as _;
use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{decode_catalog, encode_catalog};
use monumentum_core::store::storage::{FileStorage, StorageEngine};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn text_value(s: &str) -> Result<Value, DbError> {
    let text = monumentum_handler::Text::try_new(s.to_string())?;
    Ok(Value::from(text))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut id_col = ColumnDef::new("id", DataType::Integer);
    id_col.set_unique(true);

    let schema = TableSchema::try_new(
        "employees",
        vec![
            id_col,
            ColumnDef::new("name", DataType::Text),
            ColumnDef::new("salary", DataType::Float),
        ],
    )?;

    let mut cat = Catalog::new();
    cat.create_table(schema)?;

    if let Some(table) = cat.get_table_mut("employees") {
        let row1 = Row::new(vec![
            Value::from(1_i64),
            text_value("Alice")?,
            Value::try_from(7500.5_f64)?,
        ]);
        table.insert(&row1)?;

        let row2 = Row::new(vec![
            Value::from(2_i64),
            text_value("Bob")?,
            Value::try_from(6200.0_f64)?,
        ]);
        table.insert(&row2)?;
    }

    let new_name = text_value("Robert")?;
    if let Some(table) = cat.get_table_mut("employees") {
        table.set_cell(1, 1, &new_name)?;
    }

    if let Some(table) = cat.get_table("employees") {
        let found = table.lookup_by_unique(0, &Value::from(2_i64));
        assert!(found.is_some(), "Employee with id=2 should be found");
        if let Some(row) = found {
            let name = row.get(&1).and_then(Value::as_str);
            assert_eq!(name, Some("Robert"));
        }
    }

    let replacement_rows = vec![Row::new(vec![
        Value::from(3_i64),
        text_value("Charlie")?,
        Value::try_from(8000.0_f64)?,
    ])];
    if let Some(table) = cat.get_table_mut("employees") {
        table.replace_rows(replacement_rows)?;
    }

    let encoded = encode_catalog(&cat)?;
    let decoded = decode_catalog(&encoded)?;
    assert_eq!(decoded, cat, "Serialization roundtrip failed");

    let mut dir = std::env::temp_dir();
    dir.push(format!("monumentum_example_{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("example.monumentum");

    {
        let mut storage = FileStorage::open(&path, 10)?;
        storage.save_catalog(&cat)?;
        storage.checkpoint()?;
        storage.close()?;
    }

    {
        let storage = FileStorage::open(&path, 10)?;
        let reloaded = storage.get_catalog().clone();
        assert_eq!(reloaded, cat, "Reloaded catalog must match saved");
        storage.close()?;
    }

    std::fs::remove_dir_all(&dir)?;

    println!("Full features example completed successfully.");
    Ok(())
}
