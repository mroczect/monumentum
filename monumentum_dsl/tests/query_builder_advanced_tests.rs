#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction,
    clippy::arithmetic_side_effects
)]
use monumentum_core::store::storage::FileStorage;
use monumentum_dsl::{Accumulator, AggregateFunction, QueryBuilder, SumFunction};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::schema::column::{ColumnDef, DataType};
use monumentum_handler::core::schema::table_schema::TableSchema;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use monumentum_handler::traits::StorageEngine;
use tempfile::tempdir;

fn setup_numbers() -> Result<(FileStorage, tempfile::TempDir), DbError> {
    let dir = tempdir().map_err(DbError::from_io)?;
    let path = dir.path().join("numbers.db");
    let mut storage = FileStorage::open(&path, 10)?;
    let schema = TableSchema::try_new("numbers", vec![ColumnDef::new("n", DataType::Integer)])?;
    storage.create_table(schema)?;
    for i in 1..=10 {
        storage.insert_row("numbers", &Row::new(vec![Value::from(i as i64)]))?;
    }
    Ok((storage, dir))
}

#[test]
fn test_offset() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_numbers()?;
    let rows = QueryBuilder::new(&mut storage, "numbers")
        .sort_by(|a, b| {
            let va = a.get(0).and_then(Value::as_i64).unwrap_or(0);
            let vb = b.get(0).and_then(Value::as_i64).unwrap_or(0);
            va.cmp(&vb)
        })
        .limit(3)
        .offset(4)
        .execute()?;

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get(0).and_then(Value::as_i64), Some(5));
    assert_eq!(rows[1].get(0).and_then(Value::as_i64), Some(6));
    assert_eq!(rows[2].get(0).and_then(Value::as_i64), Some(7));
    Ok(())
}

fn setup_orders() -> Result<(FileStorage, tempfile::TempDir), DbError> {
    let dir = tempdir().map_err(DbError::from_io)?;
    let path = dir.path().join("orders.db");
    let mut storage = FileStorage::open(&path, 10)?;
    let schema = TableSchema::try_new(
        "orders",
        vec![
            ColumnDef::new("customer_id", DataType::Integer),
            ColumnDef::new("amount", DataType::Integer),
        ],
    )?;
    storage.create_table(schema)?;

    storage.insert_row(
        "orders",
        &Row::new(vec![Value::from(1_i64), Value::from(100_i64)]),
    )?;
    storage.insert_row(
        "orders",
        &Row::new(vec![Value::from(1_i64), Value::from(200_i64)]),
    )?;
    storage.insert_row(
        "orders",
        &Row::new(vec![Value::from(2_i64), Value::from(150_i64)]),
    )?;

    Ok((storage, dir))
}

#[test]
fn test_group_by_with_aggregate() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_orders()?;

    let groups = QueryBuilder::new(&mut storage, "orders").group_by(|row| {
        row.get(0)
            .cloned()
            .ok_or_else(|| DbError::type_mismatch("missing customer_id"))
    })?;

    let mut totals: Vec<(Value, i64)> = Vec::new();
    for (key, rows) in groups {
        let mut acc = SumFunction.init();
        for row in &rows {
            let amount = row
                .get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| DbError::type_mismatch("missing amount"))?;
            acc.update(&Value::from(amount))?;
        }
        let sum = acc.finish()?.as_i64().unwrap_or(0);
        totals.push((key, sum));
    }

    let total_customer_1 = totals
        .iter()
        .find(|(k, _)| *k == Value::from(1_i64))
        .map(|(_, v)| *v)
        .unwrap_or(0);
    let total_customer_2 = totals
        .iter()
        .find(|(k, _)| *k == Value::from(2_i64))
        .map(|(_, v)| *v)
        .unwrap_or(0);

    assert_eq!(total_customer_1, 300);
    assert_eq!(total_customer_2, 150);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct SumSquaresFunction;

impl AggregateFunction for SumSquaresFunction {
    fn name(&self) -> &'static str {
        "sum_squares"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(SumSquaresAccumulator { sum: 0.0 })
    }
}

#[derive(Debug)]
struct SumSquaresAccumulator {
    sum: f64,
}

impl Accumulator for SumSquaresAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        if let Some(i) = value.as_i64() {
            self.sum += (i * i) as f64;
        } else if let Some(f) = value.as_f64() {
            self.sum += f * f;
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        Value::try_from(self.sum)
    }
}

#[test]
fn test_custom_aggregate_in_query() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_numbers()?;
    let result =
        QueryBuilder::new(&mut storage, "numbers").aggregate(&SumSquaresFunction, |row| {
            row.get(0)
                .cloned()
                .ok_or_else(|| DbError::type_mismatch("missing n"))
        })?;

    let sum_sq = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((sum_sq - 385.0).abs() < 1e-9);
    Ok(())
}

fn setup_customers_orders() -> Result<(FileStorage, tempfile::TempDir), DbError> {
    let dir = tempdir().map_err(DbError::from_io)?;
    let path = dir.path().join("join_test.db");
    let mut storage = FileStorage::open(&path, 10)?;

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

    let orders_schema = TableSchema::try_new(
        "orders",
        vec![
            ColumnDef::new("id", DataType::Integer),
            ColumnDef::new("customer_id", DataType::Integer),
            ColumnDef::new("amount", DataType::Integer),
        ],
    )?;
    storage.create_table(orders_schema)?;
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

    Ok((storage, dir))
}

#[test]
fn test_join_inner() -> Result<(), DbError> {
    let (mut storage, _dir) = setup_customers_orders()?;

    let joined = QueryBuilder::new(&mut storage, "orders").join_inner(
        "customers",
        |row| {
            row.get(1)
                .cloned()
                .ok_or_else(|| DbError::type_mismatch("missing customer_id"))
        },
        |row| {
            row.get(0)
                .cloned()
                .ok_or_else(|| DbError::type_mismatch("missing id"))
        },
    )?;

    let rows: Vec<_> = joined.into_iter().collect();
    assert_eq!(rows.len(), 3);

    let first = &rows[0];
    let name = first.values().last().and_then(Value::as_str);
    assert!(name == Some("Alice") || name == Some("Bob"));

    Ok(())
}
