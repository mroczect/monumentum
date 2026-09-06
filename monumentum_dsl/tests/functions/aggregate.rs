use monumentum_dsl::{
    AggregateFunction, AvgFunction, CountFunction, MaxFunction, MinFunction, SumFunction,
};
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_sum_accumulator() -> Result<(), DbError> {
    let mut acc = SumFunction.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::from(2_i64))?;
    acc.update(&Value::from(3_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(6));
    Ok(())
}

#[test]
fn test_sum_ignores_null() -> Result<(), DbError> {
    let mut acc = SumFunction.init();
    acc.update(&Value::Null)?;
    acc.update(&Value::from(5_i64))?;
    acc.update(&Value::Null)?;
    acc.update(&Value::from(7_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(12));
    Ok(())
}

#[test]
fn test_sum_type_mismatch() -> Result<(), DbError> {
    let mut acc = SumFunction.init();
    let text = Value::try_from("hello".to_string())?;
    assert!(acc.update(&text).is_err());
    Ok(())
}

#[test]
fn test_avg_accumulator() -> Result<(), DbError> {
    let mut acc = AvgFunction.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::from(2_i64))?;
    acc.update(&Value::from(3_i64))?;
    let result = acc.finish()?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 2.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_avg_ignores_null() -> Result<(), DbError> {
    let mut acc = AvgFunction.init();
    acc.update(&Value::Null)?;
    acc.update(&Value::from(10_i64))?;
    acc.update(&Value::Null)?;
    acc.update(&Value::from(20_i64))?;
    let result = acc.finish()?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 15.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_avg_type_mismatch() -> Result<(), DbError> {
    let mut acc = AvgFunction.init();
    let text = Value::try_from("hello".to_string())?;
    assert!(acc.update(&text).is_err());
    Ok(())
}

#[test]
fn test_count_accumulator() -> Result<(), DbError> {
    let mut acc = CountFunction.init();
    acc.update(&Value::Null)?;
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::try_from("x".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_min_accumulator() -> Result<(), DbError> {
    let mut acc = MinFunction.init();
    acc.update(&Value::from(5_i64))?;
    acc.update(&Value::from(3_i64))?;
    acc.update(&Value::from(7_i64))?;
    acc.update(&Value::Null)?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_max_accumulator() -> Result<(), DbError> {
    let mut acc = MaxFunction.init();
    acc.update(&Value::from(5_i64))?;
    acc.update(&Value::from(3_i64))?;
    acc.update(&Value::from(7_i64))?;
    acc.update(&Value::Null)?;
    let result = acc.finish()?;
    assert_eq!(result.as_i64(), Some(7));
    Ok(())
}

#[test]
fn test_min_max_type_mismatch() -> Result<(), DbError> {
    let mut acc = MinFunction.init();
    acc.update(&Value::from(1_i64))?;
    let text = Value::try_from("a".to_string())?;
    assert!(acc.update(&text).is_err());
    Ok(())
}

#[test]
fn test_min_max_empty_result_null() -> Result<(), DbError> {
    let acc = MinFunction.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);

    let acc = MaxFunction.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}
