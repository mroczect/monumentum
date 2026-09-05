use monumentum_dsl::LowerFunction;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_lower_valid_text() -> Result<(), DbError> {
    let f = LowerFunction;
    let result = f.call(&[Value::try_from("HELLO".to_string())?])?;
    assert_eq!(result.as_str(), Some("hello"));
    Ok(())
}

#[test]
fn test_lower_missing_arg() -> Result<(), DbError> {
    let f = LowerFunction;
    let result = f.call(&[]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::InvalidOperation);
    }
    Ok(())
}

#[test]
fn test_lower_wrong_type() -> Result<(), DbError> {
    let f = LowerFunction;
    let result = f.call(&[Value::from(42_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
