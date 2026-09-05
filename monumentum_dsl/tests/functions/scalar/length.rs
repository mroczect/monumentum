use monumentum_dsl::LengthFunction;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[test]
fn test_length_text() -> Result<(), DbError> {
    let f = LengthFunction;
    let result = f.call(&[Value::try_from("hello".to_string())?])?;
    assert_eq!(result.as_i64(), Some(5));
    Ok(())
}

#[test]
fn test_length_blob() -> Result<(), DbError> {
    let f = LengthFunction;
    let blob = Value::try_from(vec![1_u8, 2, 3])?;
    let result = f.call(&[blob])?;
    assert_eq!(result.as_i64(), Some(3));
    Ok(())
}

#[test]
fn test_length_missing_arg() -> Result<(), DbError> {
    let f = LengthFunction;
    let result = f.call(&[]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::InvalidOperation);
    }
    Ok(())
}

#[test]
fn test_length_wrong_type() -> Result<(), DbError> {
    let f = LengthFunction;
    let result = f.call(&[Value::from(true)]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
