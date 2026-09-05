use monumentum_dsl::UpperFunction;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_upper_valid_text() -> Result<(), DbError> {
    let f = UpperFunction;
    let result = f.call(&[Value::try_from("hello".to_string())?])?;
    assert_eq!(result.as_str(), Some("HELLO"));
    Ok(())
}

#[test]
fn test_upper_missing_arg() -> Result<(), DbError> {
    let f = UpperFunction;
    let result = f.call(&[]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::InvalidOperation);
    }
    Ok(())
}

#[test]
fn test_upper_wrong_type() -> Result<(), DbError> {
    let f = UpperFunction;
    let result = f.call(&[Value::from(1_i64)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
