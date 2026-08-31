use monumentum_db::core::{Catalog, ColumnDef, DataType, Row, TableSchema, Value};
use monumentum_db::store::{FileStorage, StorageEngine};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("example.db");

    {
        let mut catalog = Catalog::new();
        let mut id_col = ColumnDef::new("id", DataType::Integer);
        id_col.set_primary_key(true);
        let schema = TableSchema::try_new("users", vec![id_col])?;
        catalog.create_table(schema)?;

        let mut storage = FileStorage::open(&path)?;
        storage.save_catalog(&catalog)?;
        println!("Catalog saved to {:?}", path);
    }

    {
        let mut storage = FileStorage::open(&path)?;
        let mut catalog = storage.load_catalog()?;
        {
            let table = catalog.get_table_mut("users").unwrap();
            table.insert(Row::new(vec![Value::from(1i64)]))?;
            table.insert(Row::new(vec![Value::from(2i64)]))?;
        }
        storage.save_catalog(&catalog)?;
        println!("Rows inserted and saved.");
    }

    {
        let mut storage = FileStorage::open(&path)?;
        let catalog = storage.load_catalog()?;
        if let Some(table) = catalog.get_table("users") {
            for row in table.rows() {
                println!("{:?}", row);
            }
        }
    }

    std::fs::remove_file(&path).ok();
    Ok(())
}
