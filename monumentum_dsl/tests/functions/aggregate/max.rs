#![allow(clippy::all)]
use monumentum_dsl::{AggregateFunction, MaxFunction};
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_max_basic() -> Result<(), DbError> {
    let f = MaxFunction;
    let mut acc = f.init();
    acc.update(&Value::from(5_i64))?;
    acc.update(&Value::from(3_i64))?;
    acc.update(&Value::from(8_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_max_no_values() -> Result<(), DbError> {
    let f = MaxFunction;
    let acc = f.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}
