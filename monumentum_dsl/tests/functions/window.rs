#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use monumentum_dsl::{
    CumeDistFunction, DenseRankFunction, FirstValueFunction, LagFunction, LastValueFunction,
    LeadFunction, NthValueFunction, NtileFunction, PercentRankFunction, RankFunction,
    RowNumberFunction, WindowFunction,
};
use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn make_partition(len: usize) -> Vec<Row> {
    (0..len)
        .map(|i| Row::new(vec![Value::from(i as i64)]))
        .collect()
}

fn make_order_values(vals: &[i64]) -> Vec<Option<Value>> {
    vals.iter().map(|v| Some(Value::from(*v))).collect()
}

#[test]
fn test_row_number() -> Result<(), DbError> {
    let partition = make_partition(5);
    let f = RowNumberFunction;
    assert_eq!(f.evaluate(&partition, 0, &[], &[])?, Value::from(1_i64));
    assert_eq!(f.evaluate(&partition, 4, &[], &[])?, Value::from(5_i64));
    Ok(())
}

#[test]
fn test_rank() -> Result<(), DbError> {
    let partition = make_partition(5);
    let order = make_order_values(&[10, 20, 20, 30, 40]);
    let f = RankFunction;
    assert_eq!(f.evaluate(&partition, 0, &[], &order)?, Value::from(1_i64));
    assert_eq!(f.evaluate(&partition, 1, &[], &order)?, Value::from(2_i64));
    assert_eq!(f.evaluate(&partition, 2, &[], &order)?, Value::from(2_i64));
    assert_eq!(f.evaluate(&partition, 3, &[], &order)?, Value::from(4_i64));
    Ok(())
}

#[test]
fn test_dense_rank() -> Result<(), DbError> {
    let partition = make_partition(5);
    let order = make_order_values(&[10, 20, 20, 30, 40]);
    let f = DenseRankFunction;
    assert_eq!(f.evaluate(&partition, 0, &[], &order)?, Value::from(1_i64));
    assert_eq!(f.evaluate(&partition, 1, &[], &order)?, Value::from(2_i64));
    assert_eq!(f.evaluate(&partition, 2, &[], &order)?, Value::from(2_i64));
    assert_eq!(f.evaluate(&partition, 3, &[], &order)?, Value::from(3_i64));
    Ok(())
}

#[test]
fn test_percent_rank() -> Result<(), DbError> {
    let partition = make_partition(5);
    let order = make_order_values(&[10, 20, 20, 30, 40]);
    let f = PercentRankFunction;
    let v = f.evaluate(&partition, 0, &[], &order)?;
    assert!((v.as_f64().unwrap() - 0.0).abs() < 1e-9);
    let v = f.evaluate(&partition, 3, &[], &order)?;
    assert!((v.as_f64().unwrap() - 0.75).abs() < 1e-9);
    Ok(())
}

#[test]
fn test_cume_dist() -> Result<(), DbError> {
    let partition = make_partition(5);
    let order = make_order_values(&[10, 20, 20, 30, 40]);
    let f = CumeDistFunction;
    let v = f.evaluate(&partition, 0, &[], &order)?;
    assert!((v.as_f64().unwrap() - 0.2).abs() < 1e-9);
    let v = f.evaluate(&partition, 2, &[], &order)?;
    assert!((v.as_f64().unwrap() - 0.6).abs() < 1e-9);
    Ok(())
}

#[test]
fn test_ntile() -> Result<(), DbError> {
    let partition = make_partition(5);
    let f = NtileFunction;
    let args = vec![Value::from(2_i64)];
    assert_eq!(f.evaluate(&partition, 0, &args, &[])?, Value::from(1_i64));
    assert_eq!(f.evaluate(&partition, 2, &args, &[])?, Value::from(1_i64));
    assert_eq!(f.evaluate(&partition, 3, &args, &[])?, Value::from(2_i64));
    Ok(())
}

#[test]
fn test_lag() -> Result<(), DbError> {
    let partition = make_partition(5);
    let f = LagFunction;
    let args = vec![Value::from(100_i64)];
    assert_eq!(f.evaluate(&partition, 0, &args, &[])?, Value::Null);
    assert_eq!(f.evaluate(&partition, 2, &args, &[])?, Value::from(100_i64));
    let args_with_default = vec![
        Value::from(100_i64),
        Value::from(1_i64),
        Value::from(999_i64),
    ];
    assert_eq!(
        f.evaluate(&partition, 0, &args_with_default, &[])?,
        Value::from(999_i64)
    );
    Ok(())
}

#[test]
fn test_lead() -> Result<(), DbError> {
    let partition = make_partition(5);
    let f = LeadFunction;
    let args = vec![Value::from(200_i64)];
    assert_eq!(f.evaluate(&partition, 4, &args, &[])?, Value::Null);
    assert_eq!(f.evaluate(&partition, 2, &args, &[])?, Value::from(200_i64));
    let args_with_default = vec![
        Value::from(200_i64),
        Value::from(1_i64),
        Value::from(888_i64),
    ];
    assert_eq!(
        f.evaluate(&partition, 4, &args_with_default, &[])?,
        Value::from(888_i64)
    );
    Ok(())
}

#[test]
fn test_first_last_value() -> Result<(), DbError> {
    let partition = make_partition(3);
    let f = FirstValueFunction;
    let args = vec![Value::from(10_i64)];
    assert_eq!(f.evaluate(&partition, 0, &args, &[])?, Value::from(10_i64));
    let f = LastValueFunction;
    assert_eq!(f.evaluate(&partition, 0, &args, &[])?, Value::from(10_i64));
    Ok(())
}

#[test]
fn test_nth_value() -> Result<(), DbError> {
    let partition = make_partition(3);
    let f = NthValueFunction;
    let args = vec![Value::from(5_i64), Value::from(2_i64)];
    assert_eq!(f.evaluate(&partition, 0, &args, &[])?, Value::from(5_i64));

    let args_big = vec![Value::from(5_i64), Value::from(4_i64)];
    assert_eq!(f.evaluate(&partition, 0, &args_big, &[])?, Value::Null);

    let args_zero = vec![Value::from(5_i64), Value::from(0_i64)];
    assert!(f.evaluate(&partition, 0, &args_zero, &[]).is_err());
    Ok(())
}
