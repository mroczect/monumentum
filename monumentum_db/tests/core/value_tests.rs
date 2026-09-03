use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;
use monumentum_db::types::{Blob, Float, Integer, Text};
use proptest::prelude::*;
use std::cmp::Ordering;

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

fn value_strategy() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<i64>().prop_map(Value::from),
        any::<bool>().prop_map(Value::from),
        ".*".prop_map(Value::from),
        prop::collection::vec(any::<u8>(), 0..10).prop_map(Value::from),
        any::<f64>()
            .prop_filter("must be finite", |f| f.is_finite())
            .prop_map(|f| Value::try_from(f).unwrap()),
    ]
}

#[test]
fn is_boolean_returns_true_only_for_boolean() {
    assert!(Value::Boolean(true).is_boolean());
    assert!(Value::Boolean(false).is_boolean());
    assert!(!Value::Null.is_boolean());
    assert!(!Value::from(1_i64).is_boolean());
}

#[test]
fn is_formula_returns_true_only_for_formula() {
    assert!(Value::Formula("SUM(A1:A10)".to_string()).is_formula());
    assert!(!Value::Null.is_formula());
    assert!(!Value::from("text").is_formula());
}

#[test]
fn as_boolean_returns_some_for_boolean_else_none() {
    assert_eq!(Value::Boolean(true).as_boolean(), Some(true));
    assert_eq!(Value::Boolean(false).as_boolean(), Some(false));
    assert_eq!(Value::Null.as_boolean(), None);
    assert_eq!(Value::from(1_i64).as_boolean(), None);
}

#[test]
fn as_formula_returns_some_str_for_formula_else_none() {
    let formula = Value::Formula("=A1".to_string());
    assert_eq!(formula.as_formula(), Some("=A1"));
    assert_eq!(Value::Null.as_formula(), None);
    assert_eq!(Value::from("=A1").as_formula(), None);
}

#[test]
fn into_boolean_consumes_and_returns_bool() {
    assert_eq!(Value::Boolean(true).into_boolean(), Some(true));
    assert_eq!(Value::Boolean(false).into_boolean(), Some(false));
    assert_eq!(Value::Null.into_boolean(), None);
    assert_eq!(Value::from(1_i64).into_boolean(), None);
}

#[test]
fn into_formula_consumes_and_returns_string() {
    let formula = Value::Formula("=A1".to_string());
    assert_eq!(formula.into_formula(), Some("=A1".to_string()));
    assert_eq!(Value::Null.into_formula(), None);
    assert_eq!(Value::from("text").into_formula(), None);
}

#[test]
fn as_i64_returns_some_for_integer_else_none() {
    assert_eq!(Value::from(42_i64).as_i64(), Some(42));
    assert_eq!(Value::Null.as_i64(), None);
    assert_eq!(Value::from("42").as_i64(), None);
    assert_eq!(Value::Boolean(true).as_i64(), None);
}

#[test]
fn as_f64_returns_some_for_float_or_integer_else_none() -> Result<(), DbError> {
    let f = Value::try_from(2.5)?;
    assert_eq!(f.as_f64(), Some(2.5));
    assert_eq!(Value::from(3_i64).as_f64(), Some(3.0));
    assert_eq!(Value::Null.as_f64(), None);
    assert_eq!(Value::from("2.5").as_f64(), None);
    Ok(())
}

#[test]
fn as_bool_returns_some_for_boolean_else_none() {
    assert_eq!(Value::Boolean(true).as_bool(), Some(true));
    assert_eq!(Value::Boolean(false).as_bool(), Some(false));
    assert_eq!(Value::Null.as_bool(), None);
    assert_eq!(Value::from(1_i64).as_bool(), None);
}

#[test]
fn as_str_returns_some_for_text_else_none() {
    assert_eq!(Value::from("hello").as_str(), Some("hello"));
    assert_eq!(Value::Null.as_str(), None);
    assert_eq!(Value::from(42_i64).as_str(), None);
}

#[test]
fn display_boolean_outputs_true_false() {
    assert_eq!(format!("{}", Value::Boolean(true)), "true");
    assert_eq!(format!("{}", Value::Boolean(false)), "false");
}

#[test]
fn display_formula_outputs_equals_prefixed() {
    let formula = Value::Formula("SUM(A1:A10)".to_string());
    assert_eq!(format!("{}", formula), "=SUM(A1:A10)");
}

#[test]
fn partial_ord_between_floats() -> Result<(), DbError> {
    let val1 = Value::try_from(1.0)?;
    let val2 = Value::try_from(2.0)?;
    assert!(val1 < val2);
    Ok(())
}

#[test]
fn partial_ord_between_texts() {
    let val1 = Value::from("apple");
    let val2 = Value::from("banana");
    assert!(val1 < val2);
}

#[test]
fn partial_ord_integer_vs_float_follows_variant_order() -> Result<(), DbError> {
    let int_val = Value::from(1_i64);
    let float_val = Value::try_from(2.0)?;
    assert_eq!(int_val.partial_cmp(&float_val), Some(Ordering::Less));
    assert_eq!(float_val.partial_cmp(&int_val), Some(Ordering::Greater));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn display_and_type_name_consistent(
        value in value_strategy(),
    ) {
        let type_name = value.type_name();
        let display = format!("{}", value);
        match value {
            Value::Null => prop_assert_eq!(display, "NULL"),
            Value::Integer(_) => prop_assert_eq!(type_name, "integer"),
            Value::Float(_) => prop_assert_eq!(type_name, "float"),
            Value::Text(_) => {
                prop_assert_eq!(type_name, "text");
                prop_assert!(display.starts_with('\''));
                prop_assert!(display.ends_with('\''));
            },
            Value::Blob(_) => prop_assert_eq!(type_name, "blob"),
            Value::Boolean(_) => prop_assert_eq!(type_name, "boolean"),
            Value::Formula(_) => {
                prop_assert_eq!(type_name, "formula");
                prop_assert!(display.starts_with('='));
            },
            _ => (),
        }
    }

    #[test]
    fn integer_roundtrip_via_i64(
        n in any::<i64>(),
    ) {
        let value = Value::from(n);
        prop_assert!(value.is_integer());
        prop_assert_eq!(value.as_i64(), Some(n));
        prop_assert_eq!(value.into_integer().map(|i| i.as_i64()), Some(n));
    }

    #[test]
    fn float_roundtrip_via_f64(
        f in any::<f64>().prop_filter("finite", |x| x.is_finite()),
    ) {
        let value = Value::try_from(f).unwrap();
        prop_assert!(value.is_float());
        prop_assert_eq!(value.as_f64(), Some(f));
        prop_assert_eq!(value.into_float().map(|fl| fl.as_f64()), Some(f));
    }

    #[test]
    fn text_roundtrip_via_string(
        s in ".*",
    ) {
        let value = Value::from(s.clone());
        prop_assert!(value.is_text());
        prop_assert_eq!(value.as_str(), Some(s.as_str()));
        prop_assert_eq!(value.into_text().map(|t| t.as_str().to_string()), Some(s));
    }

    #[test]
    fn blob_roundtrip_via_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..20),
    ) {
        let value = Value::from(bytes.clone());
        prop_assert!(value.is_blob());
        prop_assert_eq!(value.as_blob().map(|b| b.as_slice()), Some(bytes.as_slice()));
        prop_assert_eq!(value.into_blob().map(|b| b.as_slice().to_vec()), Some(bytes));
    }

    #[test]
    fn boolean_roundtrip_via_bool(
        b in any::<bool>(),
    ) {
        let value = Value::from(b);
        prop_assert!(value.is_boolean());
        prop_assert_eq!(value.as_bool(), Some(b));
        prop_assert_eq!(value.into_boolean(), Some(b));
    }

    #[test]
    fn formula_roundtrip_via_string(
        s in ".*",
    ) {
        let value = Value::Formula(s.clone());
        prop_assert!(value.is_formula());
        prop_assert_eq!(value.as_formula(), Some(s.as_str()));
        prop_assert_eq!(value.into_formula(), Some(s));
    }

    #[test]
    fn value_partial_ord_same_variant_reflexive(
        value in value_strategy(),
    ) {
        prop_assert_eq!(value.partial_cmp(&value), Some(std::cmp::Ordering::Equal));
    }

    #[test]
    fn value_equality_is_reflexive(
        value in value_strategy(),
    ) {
        prop_assert_eq!(value.clone(), value);
    }

    #[test]
    fn value_clone_equals_original(
        value in value_strategy(),
    ) {
        let cloned = value.clone();
        prop_assert_eq!(cloned, value);
    }
}
