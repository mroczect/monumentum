#![allow(clippy::all)]
use monumentum_dsl::{AggregateFunction, CountFunction};
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_count_multiple_values() -> Result<(), DbError> {
    let f = CountFunction;
    let mut acc = f.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::from(2_i64))?;
    acc.update(&Value::from(3_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_count_no_values() -> Result<(), DbError> {
    let f = CountFunction;
    let acc = f.init();
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(0));
    Ok(())
}
