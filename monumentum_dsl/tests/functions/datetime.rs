#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::arithmetic_side_effects
)]
use monumentum_dsl::FunctionRegistry;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn get_scalar<'a>(
    registry: &'a FunctionRegistry,
    name: &str,
) -> Option<&'a dyn monumentum_dsl::ScalarFunction> {
    registry.get_scalar(name)
}

#[test]
fn test_date_now() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[])?;
    let s = result
        .as_str()
        .ok_or_else(|| DbError::type_mismatch("expected text"))?;
    assert!(s.len() == 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-');
    Ok(())
}

#[test]
fn test_date_iso_basic() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[Value::try_from("2024-03-05")?])?;
    assert_eq!(result.as_str(), Some("2024-03-05"));
    Ok(())
}

#[test]
fn test_date_start_of_month() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[
        Value::try_from("2024-02-15")?,
        Value::try_from("start of month")?,
    ])?;
    assert_eq!(result.as_str(), Some("2024-02-01"));
    Ok(())
}

#[test]
fn test_date_add_days() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[Value::try_from("2024-01-01")?, Value::try_from("+1 day")?])?;
    assert_eq!(result.as_str(), Some("2024-01-02"));
    Ok(())
}

#[test]
fn test_date_add_months() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[Value::try_from("2024-01-31")?, Value::try_from("+1 month")?])?;
    assert_eq!(result.as_str(), Some("2024-02-29"));
    Ok(())
}

#[test]
fn test_datetime_now() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "datetime").expect("datetime not found");
    let result = f.call(&[])?;
    let s = result
        .as_str()
        .ok_or_else(|| DbError::type_mismatch("expected text"))?;
    assert_eq!(s.len(), 19);
    assert_eq!(s.as_bytes()[10], b' ');
    Ok(())
}

#[test]
fn test_datetime_iso_with_time() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "datetime").expect("datetime not found");
    let result = f.call(&[Value::try_from("2024-03-05T12:34:56")?])?;
    assert_eq!(result.as_str(), Some("2024-03-05 12:34:56"));
    Ok(())
}

#[test]
fn test_datetime_subsec() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "datetime").expect("datetime not found");
    let result = f.call(&[
        Value::try_from("2024-03-05T12:34:56.789")?,
        Value::try_from("subsec")?,
    ])?;
    assert_eq!(result.as_str(), Some("2024-03-05 12:34:56.789"));
    Ok(())
}

#[test]
fn test_julianday_epoch() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "julianday").expect("julianday not found");
    let result = f.call(&[Value::try_from("2000-01-01")?])?;
    let jd = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((jd - 2451544.5).abs() < 1e-9);
    Ok(())
}

#[test]
fn test_strftime_date() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "strftime").expect("strftime not found");
    let result = f.call(&[Value::try_from("%Y-%m-%d")?, Value::try_from("2024-03-05")?])?;
    assert_eq!(result.as_str(), Some("2024-03-05"));
    Ok(())
}

#[test]
fn test_strftime_time() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "strftime").expect("strftime not found");
    let result = f.call(&[Value::try_from("%H:%M")?, Value::try_from("12:34:56")?])?;
    assert_eq!(result.as_str(), Some("12:34"));
    Ok(())
}

#[test]
fn test_strftime_weekday() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "strftime").expect("strftime not found");
    let result = f.call(&[Value::try_from("%w")?, Value::try_from("2024-03-05")?])?;
    assert_eq!(result.as_str(), Some("2"));
    Ok(())
}

#[test]
fn test_time_basic() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "time").expect("time not found");
    let result = f.call(&[Value::try_from("2024-03-05T12:34:56")?])?;
    assert_eq!(result.as_str(), Some("12:34:56"));
    Ok(())
}

#[test]
fn test_time_subsec() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "time").expect("time not found");
    let result = f.call(&[Value::try_from("12:34:56.789")?, Value::try_from("subsec")?])?;
    assert_eq!(result.as_str(), Some("12:34:56.789"));
    Ok(())
}

#[test]
fn test_timediff_positive() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "timediff").expect("timediff not found");
    let result = f.call(&[
        Value::try_from("2024-03-05")?,
        Value::try_from("2024-03-04")?,
    ])?;
    assert_eq!(result.as_str(), Some("+0000-00-01 00:00:00.000"));
    Ok(())
}

#[test]
fn test_timediff_negative() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "timediff").expect("timediff not found");
    let result = f.call(&[
        Value::try_from("2024-03-04")?,
        Value::try_from("2024-03-05")?,
    ])?;
    assert_eq!(result.as_str(), Some("-0000-00-01 00:00:00.000"));
    Ok(())
}

#[test]
fn test_unixepoch_epoch() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "unixepoch").expect("unixepoch not found");
    let result = f.call(&[Value::try_from("1970-01-01")?])?;
    let ts = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert_eq!(ts, 0.0);
    Ok(())
}

#[test]
fn test_unixepoch_subsec() -> Result<(), DbError> {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "unixepoch").expect("unixepoch not found");
    let result = f.call(&[
        Value::try_from("1970-01-01T00:00:01.5")?,
        Value::try_from("subsec")?,
    ])?;
    let ts = result
        .as_f64()
        .ok_or_else(|| DbError::type_mismatch("expected float"))?;
    assert!((ts - 1.5).abs() < 1e-9);
    Ok(())
}

#[test]
fn test_invalid_format() {
    let registry = FunctionRegistry::new();
    let f = get_scalar(&registry, "date").expect("date not found");
    let result = f.call(&[Value::try_from("not-a-date").unwrap()]);
    assert!(result.is_err());
}
