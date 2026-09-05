#![allow(clippy::all)]

use monumentum_dsl::{AggregateFunction, AvgFunction};
use monumentum_handler::MonumentumError;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_avg_basic() -> Result<(), DbError> {
    let f = AvgFunction;
    let mut acc = f.init();
    acc.update(&Value::from(10_i64))?;
    acc.update(&Value::from(20_i64))?;
    acc.update(&Value::from(30_i64))?;
    let result = acc.finish()?;
    if let Value::Float(avg) = result {
        assert!((avg.as_f64() - 20.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_avg_no_values() -> Result<(), DbError> {
    let f = AvgFunction;
    let acc = f.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}

#[test]
fn test_avg_wrong_type() -> Result<(), DbError> {
    let f = AvgFunction;
    let mut acc = f.init();
    let result = acc.update(&Value::from(true));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
