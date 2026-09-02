use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::types::{Blob, Float, Integer, Text};

fn float_value(value: f64) -> Result<Value, DbError> {
    Value::try_from(value)
}

#[test]
fn default_is_null() {
    let val = Value::default();
    assert!(val.is_null());
    assert_eq!(val.type_name(), "null");
}

#[test]
fn is_null_returns_true_only_for_null() -> Result<(), DbError> {
    assert!(Value::Null.is_null());
    assert!(!Value::from(1_i64).is_null());
    assert!(!Value::from("text").is_null());
    let f = float_value(1.0)?;
    assert!(!f.is_null());
    Ok(())
}

#[test]
fn is_integer_returns_true_only_for_integer() -> Result<(), DbError> {
    assert!(Value::from(1_i64).is_integer());
    assert!(!Value::Null.is_integer());
    let f = float_value(1.0)?;
    assert!(!f.is_integer());
    assert!(!Value::from("text").is_integer());
    Ok(())
}

#[test]
fn is_float_returns_true_only_for_float() -> Result<(), DbError> {
    let f = float_value(1.0)?;
    assert!(f.is_float());
    assert!(!Value::Null.is_float());
    assert!(!Value::from(1_i64).is_float());
    assert!(!Value::from("text").is_float());
    Ok(())
}

#[test]
fn is_text_returns_true_only_for_text() -> Result<(), DbError> {
    assert!(Value::from("text").is_text());
    assert!(!Value::Null.is_text());
    assert!(!Value::from(1_i64).is_text());
    let f = float_value(1.0)?;
    assert!(!f.is_text());
    Ok(())
}

#[test]
fn is_blob_returns_true_only_for_blob() -> Result<(), DbError> {
    let blob = vec![1_u8, 2, 3];
    assert!(Value::from(blob).is_blob());
    assert!(!Value::Null.is_blob());
    assert!(!Value::from(1_i64).is_blob());
    assert!(!Value::from("text").is_blob());
    Ok(())
}

#[test]
fn type_name_returns_correct_strings() -> Result<(), DbError> {
    assert_eq!(Value::Null.type_name(), "null");
    assert_eq!(Value::from(1_i64).type_name(), "integer");
    let f = float_value(1.0)?;
    assert_eq!(f.type_name(), "float");
    assert_eq!(Value::from("text").type_name(), "text");
    assert_eq!(Value::from(vec![1_u8]).type_name(), "blob");
    Ok(())
}

#[test]
fn as_integer_returns_some_for_integer_else_none() -> Result<(), DbError> {
    let val = Value::from(42_i64);
    let int = val
        .as_integer()
        .ok_or_else(|| DbError::invalid_operation("expected integer"))?;
    assert_eq!(int.as_i64(), 42);
    assert!(Value::Null.as_integer().is_none());
    let f = float_value(1.0)?;
    assert!(f.as_integer().is_none());
    Ok(())
}

#[test]
fn as_float_returns_some_for_float_else_none() -> Result<(), DbError> {
    let val = float_value(2.5)?;
    let float = val
        .as_float()
        .ok_or_else(|| DbError::invalid_operation("expected float"))?;
    assert_eq!(float.as_f64(), 2.5);
    assert!(Value::Null.as_float().is_none());
    assert!(Value::from(1_i64).as_float().is_none());
    Ok(())
}

#[test]
fn as_text_returns_some_for_text_else_none() -> Result<(), DbError> {
    let val = Value::from("hello");
    let text = val
        .as_text()
        .ok_or_else(|| DbError::invalid_operation("expected text"))?;
    assert_eq!(text.as_str(), "hello");
    assert!(Value::Null.as_text().is_none());
    assert!(Value::from(1_i64).as_text().is_none());
    Ok(())
}

#[test]
fn as_blob_returns_some_for_blob_else_none() -> Result<(), DbError> {
    let data = vec![1_u8, 2, 3];
    let val = Value::from(data.clone());
    let blob = val
        .as_blob()
        .ok_or_else(|| DbError::invalid_operation("expected blob"))?;
    assert_eq!(blob.as_slice(), data.as_slice());
    assert!(Value::Null.as_blob().is_none());
    assert!(Value::from(1_i64).as_blob().is_none());
    Ok(())
}

#[test]
fn into_integer_consumes_and_returns_integer_else_none() -> Result<(), DbError> {
    let val = Value::from(42_i64);
    let int = val
        .into_integer()
        .ok_or_else(|| DbError::invalid_operation("expected integer"))?;
    assert_eq!(int.as_i64(), 42);
    assert!(Value::Null.into_integer().is_none());
    Ok(())
}

#[test]
fn into_float_consumes_and_returns_float_else_none() -> Result<(), DbError> {
    let val = float_value(2.5)?;
    let float = val
        .into_float()
        .ok_or_else(|| DbError::invalid_operation("expected float"))?;
    assert_eq!(float.as_f64(), 2.5);
    assert!(Value::Null.into_float().is_none());
    Ok(())
}

#[test]
fn into_text_consumes_and_returns_text_else_none() -> Result<(), DbError> {
    let val = Value::from("hello");
    let text = val
        .into_text()
        .ok_or_else(|| DbError::invalid_operation("expected text"))?;
    assert_eq!(text.as_str(), "hello");
    assert!(Value::Null.into_text().is_none());
    Ok(())
}

#[test]
fn into_blob_consumes_and_returns_blob_else_none() -> Result<(), DbError> {
    let data = vec![1_u8, 2, 3];
    let val = Value::from(data.clone());
    let blob = val
        .into_blob()
        .ok_or_else(|| DbError::invalid_operation("expected blob"))?;
    assert_eq!(blob.as_slice(), data.as_slice());
    assert!(Value::Null.into_blob().is_none());
    Ok(())
}

#[test]
fn display_null_outputs_null() {
    assert_eq!(format!("{}", Value::Null), "NULL");
}

#[test]
fn display_integer_outputs_number() {
    assert_eq!(format!("{}", Value::from(42_i64)), "42");
}

#[test]
fn display_float_outputs_debug_format() -> Result<(), DbError> {
    let val = float_value(2.5)?;
    let expected = format!("{:?}", 2.5_f64);
    assert_eq!(format!("{}", val), expected);
    Ok(())
}

#[test]
fn display_text_quotes_and_escapes_single_quotes() {
    assert_eq!(format!("{}", Value::from("hello")), "'hello'");
    assert_eq!(format!("{}", Value::from("it's")), "'it''s'");
}

#[test]
fn display_blob_uses_blob_display() {
    let blob = Blob::new(vec![1, 2, 3]);
    let val = Value::from(blob);
    assert_eq!(format!("{}", val), "Blob(3 bytes)");
}

#[test]
fn from_unit_creates_null() {
    let val = Value::from(());
    assert!(val.is_null());
}

#[test]
fn from_integer_creates_integer_value() {
    let int = Integer::new(123);
    let val = Value::from(int);
    assert!(val.is_integer());
    if let Value::Integer(i) = val {
        assert_eq!(i.as_i64(), 123);
    }
}

#[test]
fn from_float_creates_float_value() -> Result<(), DbError> {
    let float = Float::try_new(2.5)?;
    let val = Value::from(float);
    assert!(val.is_float());
    if let Value::Float(f) = val {
        assert_eq!(f.as_f64(), 2.5);
    }
    Ok(())
}

#[test]
fn from_text_creates_text_value() {
    let text = Text::new("hello".to_string());
    let val = Value::from(text);
    assert!(val.is_text());
    if let Value::Text(t) = val {
        assert_eq!(t.as_str(), "hello");
    }
}

#[test]
fn from_blob_creates_blob_value() {
    let blob = Blob::new(vec![1, 2, 3]);
    let val = Value::from(blob);
    assert!(val.is_blob());
    if let Value::Blob(b) = val {
        assert_eq!(b.as_slice(), &[1, 2, 3]);
    }
}

#[test]
fn from_i64_creates_integer_value() {
    let val = Value::from(42_i64);
    assert!(val.is_integer());
}

#[test]
fn try_from_f64_creates_float_value_for_finite() -> Result<(), DbError> {
    let val = Value::try_from(2.5)?;
    assert!(val.is_float());
    if let Value::Float(f) = val {
        assert_eq!(f.as_f64(), 2.5);
    }
    Ok(())
}

#[test]
fn try_from_f64_returns_error_for_nan() {
    let result = Value::try_from(f64::NAN);
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e.to_string(),
            "Type mismatch: float must be finite (no NaN or infinity)"
        );
    }
}

#[test]
fn try_from_f64_returns_error_for_infinity() {
    let result = Value::try_from(f64::INFINITY);
    assert!(result.is_err());
}

#[test]
fn from_string_creates_text_value() {
    let val = Value::from("hello".to_string());
    assert!(val.is_text());
}

#[test]
fn from_str_creates_text_value() {
    let val = Value::from("hello");
    assert!(val.is_text());
}

#[test]
fn from_vec_u8_creates_blob_value() {
    let data = vec![1, 2, 3];
    let val = Value::from(data.clone());
    assert!(val.is_blob());
    if let Value::Blob(b) = val {
        assert_eq!(b.as_slice(), data.as_slice());
    }
}

#[test]
fn from_slice_u8_creates_blob_value() {
    let data = [1, 2, 3];
    let val = Value::from(&data[..]);
    assert!(val.is_blob());
    if let Value::Blob(b) = val {
        assert_eq!(b.as_slice(), &data);
    }
}

#[test]
fn equality_between_same_variants() {
    assert_eq!(Value::from(1_i64), Value::from(1_i64));
    assert_eq!(Value::from("text"), Value::from("text"));
}

#[test]
fn partial_ord_between_integers() {
    let val1 = Value::from(1_i64);
    let val2 = Value::from(2_i64);
    assert!(val1 < val2);
}
