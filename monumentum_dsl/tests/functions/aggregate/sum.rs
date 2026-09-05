#![allow(clippy::all)]
use monumentum_dsl::{AggregateFunction, SumFunction};
use monumentum_handler::MonumentumError;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_sum_positive() -> Result<(), DbError> {
    let f = SumFunction;
    let mut acc = f.init();
    acc.update(&Value::from(10_i64))?;
    acc.update(&Value::from(20_i64))?;
    acc.update(&Value::from(30_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(60));
    Ok(())
}

#[test]
fn test_sum_no_values() -> Result<(), DbError> {
    let f = SumFunction;
    let acc = f.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}

#[test]
fn test_sum_wrong_type() -> Result<(), DbError> {
    let f = SumFunction;
    let mut acc = f.init();
    let result = acc.update(&Value::try_from("hello".to_string())?);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
