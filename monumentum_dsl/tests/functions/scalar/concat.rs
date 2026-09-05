#![allow(clippy::all)]
use monumentum_dsl::ConcatFunction;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use monumentum_dsl::ScalarFunction;
use monumentum_handler::MonumentumError;
#[test]
fn test_concat_two_texts() -> Result<(), DbError> {
    let f = ConcatFunction;
    let arg1 = Value::try_from("Hello".to_string())?;
    let arg2 = Value::try_from("World".to_string())?;
    let result = f.call(&[arg1, arg2])?;
    assert_eq!(result.as_str(), Some("HelloWorld"));
    Ok(())
}

#[test]
fn test_concat_wrong_type() -> Result<(), DbError> {
    let f = ConcatFunction;
    let arg1 = Value::try_from("Hello".to_string())?;
    let arg2 = Value::from(42_i64);
    let result = f.call(&[arg1, arg2]);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), monumentum_handler::error::ErrorKind::TypeMismatch);
    }
    Ok(())
}
