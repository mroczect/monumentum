use super::{call_function, float_value, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

#[test]
fn sum_integers() {
    let result = unwrap_result(call_function(
        "SUM",
        &[
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(2)),
        ],
    ));
    assert_eq!(result, Value::Integer(Integer::new(3)));
}

#[test]
fn sum_floats() {
    let result = unwrap_result(call_function("SUM", &[float_value(1.5), float_value(2.5)]));
    assert_eq!(result, float_value(4.0));
}

#[test]
fn sum_mixed() {
    let result = unwrap_result(call_function(
        "SUM",
        &[Value::Integer(Integer::new(1)), float_value(2.5)],
    ));
    assert_eq!(result, float_value(3.5));
}

#[test]
fn sum_empty_args_errors() {
    assert!(matches!(
        call_function("SUM", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn sum_non_numeric_errors() {
    assert!(matches!(
        call_function("SUM", &[Value::Text("abc".to_string().into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
