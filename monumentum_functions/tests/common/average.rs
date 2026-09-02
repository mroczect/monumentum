use super::{call_function, float_value, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

#[test]
fn average_integers_returns_float() {
    let result = unwrap_result(call_function(
        "AVERAGE",
        &[
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(2)),
        ],
    ));
    assert_eq!(result, float_value(1.5));
}

#[test]
fn average_single_float() {
    let result = unwrap_result(call_function("AVERAGE", &[float_value(5.0)]));
    assert_eq!(result, float_value(5.0));
}

#[test]
fn average_empty_errors() {
    assert!(matches!(
        call_function("AVERAGE", &[]),
        Err(FormulaError::Eval(_))
    ));
}

#[test]
fn average_non_numeric_errors() {
    assert!(matches!(
        call_function("AVERAGE", &[Value::Boolean(true)]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
