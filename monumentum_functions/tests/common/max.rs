use super::{call_function, float_value, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

#[test]
fn max_integers() {
    let result = unwrap_result(call_function(
        "MAX",
        &[
            Value::Integer(Integer::new(5)),
            Value::Integer(Integer::new(2)),
            Value::Integer(Integer::new(8)),
        ],
    ));
    assert_eq!(result, Value::Integer(Integer::new(8)));
}

#[test]
fn max_floats() {
    let result = unwrap_result(call_function("MAX", &[float_value(1.5), float_value(3.5)]));
    assert_eq!(result, float_value(3.5));
}

#[test]
fn max_mixed_returns_float() {
    let result = unwrap_result(call_function(
        "MAX",
        &[Value::Integer(Integer::new(3)), float_value(4.5)],
    ));
    assert_eq!(result, float_value(4.5));
}

#[test]
fn max_empty_errors() {
    assert!(matches!(
        call_function("MAX", &[]),
        Err(FormulaError::Eval(_))
    ));
}

#[test]
fn max_non_numeric_errors() {
    assert!(matches!(
        call_function("MAX", &[Value::Boolean(true)]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
