#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
use monumentum_dsl::FunctionRegistry;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn call_scalar(reg: &FunctionRegistry, name: &str, args: &[Value]) -> Result<Value, DbError> {
    let f = reg
        .get_scalar(name)
        .ok_or_else(|| DbError::unsupported(name))?;
    f.call(args)
}

#[test]
fn test_scalar_functions_registry_presence() {
    let reg = FunctionRegistry::new();
    for name in ["upper", "lower", "length", "concat"] {
        assert!(reg.get_scalar(name).is_some(), "missing scalar {}", name);
    }
}

#[test]
fn test_upper_basic() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "upper", &[Value::try_from("hello")?])?;
    assert_eq!(result.as_str(), Some("HELLO"));
    Ok(())
}

#[test]
fn test_upper_empty_string() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "upper", &[Value::try_from("")?])?;
    assert_eq!(result.as_str(), Some(""));
    Ok(())
}

#[test]
fn test_lower_basic() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "lower", &[Value::try_from("WORLD")?])?;
    assert_eq!(result.as_str(), Some("world"));
    Ok(())
}

#[test]
fn test_lower_empty_string() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "lower", &[Value::try_from("")?])?;
    assert_eq!(result.as_str(), Some(""));
    Ok(())
}

#[test]
fn test_length_text_ascii() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "length", &[Value::try_from("hello")?])?;
    assert_eq!(result.as_i64(), Some(5));
    Ok(())
}

#[test]
fn test_length_text_unicode() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "length", &[Value::try_from("héllo")?])?;
    assert_eq!(result.as_i64(), Some(5));
    Ok(())
}

#[test]
fn test_length_text_empty() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "length", &[Value::try_from("")?])?;
    assert_eq!(result.as_i64(), Some(0));
    Ok(())
}

#[test]
fn test_length_type_mismatch_integer() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "length", &[Value::from(123_i64)]);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_length_type_mismatch_null() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "length", &[Value::Null]);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_concat_basic() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(
        &reg,
        "concat",
        &[
            Value::try_from("foo")?,
            Value::try_from("bar")?,
            Value::try_from("baz")?,
        ],
    )?;
    assert_eq!(result.as_str(), Some("foobarbaz"));
    Ok(())
}

#[test]
fn test_concat_empty_args() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "concat", &[])?;
    assert_eq!(result.as_str(), Some(""));
    Ok(())
}

#[test]
fn test_concat_single_arg() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(&reg, "concat", &[Value::try_from("solo")?])?;
    assert_eq!(result.as_str(), Some("solo"));
    Ok(())
}

#[test]
fn test_concat_type_mismatch() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let result = call_scalar(
        &reg,
        "concat",
        &[Value::try_from("ok")?, Value::from(42_i64)],
    );
    assert!(result.is_err());
    Ok(())
}
