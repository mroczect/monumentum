#![allow(clippy::all)]

use monumentum_dsl::{
    AggregateFunction, GroupConcatFunction, MedianFunction, PercentileContFunction,
    PercentileDiscFunction, StringAggFunction, TotalFunction,
};
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_group_concat_basic() -> Result<(), DbError> {
    let mut acc = GroupConcatFunction::new(", ").init();
    acc.update(&Value::try_from("a".to_string())?)?;
    acc.update(&Value::try_from("b".to_string())?)?;
    acc.update(&Value::try_from("c".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("a, b, c"));
    Ok(())
}

#[test]
fn test_group_concat_ignores_null() -> Result<(), DbError> {
    let mut acc = GroupConcatFunction::new(",").init();
    acc.update(&Value::Null)?;
    acc.update(&Value::try_from("x".to_string())?)?;
    acc.update(&Value::Null)?;
    acc.update(&Value::try_from("y".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("x,y"));
    Ok(())
}

#[test]
fn test_group_concat_type_mismatch() {
    let mut acc = GroupConcatFunction::new(",").init();
    let result = acc.update(&Value::from(42_i64));
    assert!(result.is_err());
}

#[test]
fn test_string_agg_same_as_group_concat() -> Result<(), DbError> {
    let mut acc = StringAggFunction::new("-").init();
    acc.update(&Value::try_from("a".to_string())?)?;
    acc.update(&Value::try_from("b".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("a-b"));
    Ok(())
}

#[test]
fn test_total_integer() -> Result<(), DbError> {
    let mut acc = TotalFunction.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::from(2_i64))?;
    acc.update(&Value::from(3_i64))?;
    let result = acc.finish()?;
    assert_eq!(result.as_f64(), Some(6.0));
    Ok(())
}

#[test]
fn test_total_float_and_mixed() -> Result<(), DbError> {
    let mut acc = TotalFunction.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::try_from(2.5_f64)?)?;
    acc.update(&Value::Null)?;
    acc.update(&Value::try_from("text".to_string())?)?;
    let result = acc.finish()?;
    let v = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((v - 3.5).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_total_no_values_returns_zero() -> Result<(), DbError> {
    let acc = TotalFunction.init();
    let result = acc.finish()?;
    assert_eq!(result.as_f64(), Some(0.0));
    Ok(())
}

#[test]
fn test_median_odd_count() -> Result<(), DbError> {
    let mut acc = MedianFunction.init();
    for v in [5_i64, 2, 8, 1, 9] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let median = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((median - 5.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_median_even_count() -> Result<(), DbError> {
    let mut acc = MedianFunction.init();
    for v in [1_i64, 2, 3, 4] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let median = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((median - 2.5).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_percentile_cont_quartile() -> Result<(), DbError> {
    let mut acc = PercentileContFunction::new(0.25).init();
    for v in [1_i64, 2, 3, 4, 5] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let val = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((val - 2.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_percentile_cont_median() -> Result<(), DbError> {
    let mut acc = PercentileContFunction::new(0.5).init();
    for v in [10_i64, 20, 30] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let val = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((val - 20.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_percentile_cont_out_of_range_p() {
    let mut acc = PercentileContFunction::new(1.5).init();
    assert!(acc.update(&Value::from(1_i64)).is_ok());
    let result = acc.finish();
    assert!(result.is_err());
}

#[test]
fn test_percentile_cont_type_mismatch() -> Result<(), DbError> {
    let mut acc = PercentileContFunction::new(0.5).init();
    let result = acc.update(&Value::try_from("not a number".to_string())?);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_percentile_disc_quartile() -> Result<(), DbError> {
    let mut acc = PercentileDiscFunction::new(0.25).init();
    for v in [1_i64, 2, 3, 4, 5] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let val = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((val - 2.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_percentile_disc_median_odd() -> Result<(), DbError> {
    let mut acc = PercentileDiscFunction::new(0.5).init();
    for v in [100_i64, 200, 300] {
        acc.update(&Value::from(v))?;
    }
    let result = acc.finish()?;
    let val = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((val - 200.0).abs() < 1e-12);
    Ok(())
}

#[test]
fn test_percentile_disc_no_values_returns_null() -> Result<(), DbError> {
    let acc = PercentileDiscFunction::new(0.5).init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}
