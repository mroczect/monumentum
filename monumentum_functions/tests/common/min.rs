use super::{call_function, float_value, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

#[test]
fn min_integers() {
    let result = unwrap_result(call_function(
        "MIN",
        &[
            Value::Integer(Integer::new(5)),
            Value::Integer(Integer::new(2)),
            Value::Integer(Integer::new(8)),
        ],
    ));
    assert_eq!(result, Value::Integer(Integer::new(2)));
}

#[test]
fn min_floats() {
    let result = unwrap_result(call_function("MIN", &[float_value(1.5), float_value(0.5)]));
    assert_eq!(result, float_value(0.5));
}

#[test]
fn min_mixed_returns_float() {
    let result = unwrap_result(call_function(
        "MIN",
        &[Value::Integer(Integer::new(3)), float_value(2.5)],
    ));
    assert_eq!(result, float_value(2.5));
}

#[test]
fn min_empty_errors() {
    assert!(matches!(
        call_function("MIN", &[]),
        Err(FormulaError::Eval(_))
    ));
}

#[test]
fn min_non_numeric_errors() {
    assert!(matches!(
        call_function("MIN", &[Value::Text("a".to_string().into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
