use core::error::Error;
use fs2 as _;
use monumentum_core::catalog::Catalog;
use monumentum_core::serde::{decode_catalog, encode_catalog};
use monumentum_core::store::storage::FileStorage;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;

fn main() -> Result<(), Box<dyn Error>> {
    let schema = TableSchema::try_new(
        "employees",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
            ColumnDef::new("salary", DataType::Float),
        ],
    )?;

    let mut cat = Catalog::new();
    cat.create_table(schema)?;

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
