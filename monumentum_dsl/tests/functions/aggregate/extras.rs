#![allow(clippy::all)]
use monumentum_dsl::{
    AggregateFunction, GroupConcatFunction, MedianFunction, PercentileContFunction,
    PercentileDiscFunction, StringAggFunction, TotalFunction,
};
use monumentum_handler::MonumentumError;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
#[test]
fn test_group_concat_basic() -> Result<(), DbError> {
    let f = GroupConcatFunction::new(",");
    let mut acc = f.init();
    acc.update(&Value::try_from("a".to_string())?)?;
    acc.update(&Value::try_from("b".to_string())?)?;
    acc.update(&Value::try_from("c".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("a,b,c"));
    Ok(())
}

#[test]
fn test_group_concat_custom_separator() -> Result<(), DbError> {
    let f = GroupConcatFunction::new("|");
    let mut acc = f.init();
    acc.update(&Value::try_from("x".to_string())?)?;
    acc.update(&Value::try_from("y".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("x|y"));
    Ok(())
}

#[test]
fn test_group_concat_wrong_type() -> Result<(), DbError> {
    let f = GroupConcatFunction::new(",");
    let mut acc = f.init();
    let result = acc.update(&Value::from(42_i64));
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}

#[test]
fn test_string_agg_basic() -> Result<(), DbError> {
    let f = StringAggFunction::new("-");
    let mut acc = f.init();
    acc.update(&Value::try_from("apple".to_string())?)?;
    acc.update(&Value::try_from("banana".to_string())?)?;
    let result = acc.finish()?;
    assert_eq!(result.as_str(), Some("apple-banana"));
    Ok(())
}

#[test]
fn test_total_integers() -> Result<(), DbError> {
    let f = TotalFunction;
    let mut acc = f.init();
    acc.update(&Value::from(1_i64))?;
    acc.update(&Value::from(2_i64))?;
    acc.update(&Value::from(3_i64))?;
    let result = acc.finish()?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 6.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_total_mixed_and_ignores_non_numeric() -> Result<(), DbError> {
    let f = TotalFunction;
    let mut acc = f.init();
    acc.update(&Value::from(10_i64))?;
    acc.update(&Value::try_from(5.5_f64)?)?;
    acc.update(&Value::Null)?;
    acc.update(&Value::try_from("ignore".to_string())?)?;
    acc.update(&Value::from(true))?;
    let result = acc.finish()?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 15.5).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_total_no_values() -> Result<(), DbError> {
    let f = TotalFunction;
    let acc = f.init();
    let result = acc.finish()?;
    if let Value::Float(f) = result {
        assert!((f.as_f64() - 0.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_median_odd_count() -> Result<(), DbError> {
    let f = MedianFunction;
    let mut acc = f.init();
    acc.update(&Value::try_from(2.0_f64)?)?;
    acc.update(&Value::try_from(1.0_f64)?)?;
    acc.update(&Value::try_from(3.0_f64)?)?;
    let result = acc.finish()?;
    if let Value::Float(median) = result {
        assert!((median.as_f64() - 2.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_median_even_count() -> Result<(), DbError> {
    let f = MedianFunction;
    let mut acc = f.init();
    acc.update(&Value::try_from(1.0_f64)?)?;
    acc.update(&Value::try_from(2.0_f64)?)?;
    acc.update(&Value::try_from(3.0_f64)?)?;
    acc.update(&Value::try_from(4.0_f64)?)?;
    let result = acc.finish()?;
    if let Value::Float(median) = result {
        assert!((median.as_f64() - 2.5).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_median_empty() -> Result<(), DbError> {
    let f = MedianFunction;
    let acc = f.init();
    let result = acc.finish()?;
    assert_eq!(result, Value::Null);
    Ok(())
}

#[test]
fn test_percentile_cont_basic() -> Result<(), DbError> {
    let f = PercentileContFunction::new(0.5);
    let mut acc = f.init();
    for i in 1..=5 {
        acc.update(&Value::try_from(i as f64)?)?;
    }
    let result = acc.finish()?;
    if let Value::Float(p) = result {
        assert!((p.as_f64() - 3.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_percentile_disc_basic() -> Result<(), DbError> {
    let f = PercentileDiscFunction::new(0.5);
    let mut acc = f.init();
    for i in 1..=5 {
        acc.update(&Value::try_from(i as f64)?)?;
    }
    let result = acc.finish()?;
    if let Value::Float(p) = result {
        assert!((p.as_f64() - 3.0).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_percentile_invalid_p() -> Result<(), DbError> {
    let f = PercentileContFunction::new(1.5);
    let mut acc = f.init();
    acc.update(&Value::try_from(1.0_f64)?)?;
    let result = acc.finish();
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.kind(),
            monumentum_handler::error::ErrorKind::InvalidOperation
        );
    }
    Ok(())
}
