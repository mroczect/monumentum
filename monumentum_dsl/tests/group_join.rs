use core::error::Error;
use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::QueryBuilder;
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use tempfile as _;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join("group_join.db");
    let mut storage = FileStorage::open(&path, 10)?;

    let orders_schema = TableSchema::try_new(
        "orders",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("customer_id", DataType::Integer),
            ColumnDef::new("amount", DataType::Integer),
        ],
    )?;
    storage.create_table(orders_schema)?;

    let customers_schema = TableSchema::try_new(
        "customers",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("name", DataType::Text),
        ],
    )?;
    storage.create_table(customers_schema)?;

    storage.insert_row(
        "customers",
        &Row::new(vec![
            Value::from(1_i64),
            Value::try_from("Alice".to_string())?,
        ]),
    )?;
    storage.insert_row(
        "customers",
        &Row::new(vec![
            Value::from(2_i64),
            Value::try_from("Bob".to_string())?,
        ]),
    )?;

    storage.insert_row(
        "orders",
        &Row::new(vec![
            Value::from(100_i64),
            Value::from(1_i64),
            Value::from(250_i64),
        ]),
    )?;
    storage.insert_row(
        "orders",
        &Row::new(vec![
            Value::from(101_i64),
            Value::from(2_i64),
            Value::from(500_i64),
        ]),
    )?;
    storage.insert_row(
        "orders",
        &Row::new(vec![
            Value::from(102_i64),
            Value::from(1_i64),
            Value::from(750_i64),
        ]),
    )?;

    let joined = QueryBuilder::new(&mut storage, "orders").join_inner(
        "customers",
        |row| {
            let value = row
                .get(1)
                .ok_or_else(|| DbError::type_mismatch("missing customer_id"))?
                .clone();
            Ok(value)
        },
        |row| {
            let value = row
                .get(0)
                .ok_or_else(|| DbError::type_mismatch("missing id"))?
                .clone();
            Ok(value)
        },
    )?;

    for row in joined {
        println!("{:?}", row);
    }

    let groups = QueryBuilder::new(&mut storage, "orders").group_by(|row| {
        let key = row
            .get(1)
            .ok_or_else(|| DbError::type_mismatch("missing customer_id"))?
            .clone();
        Ok(key)
    })?;

    for (key, rows) in groups {
        println!("Customer {:?} has {} orders", key, rows.len());
    }

    storage.checkpoint()?;
    storage.close()?;
    Ok(())
}
