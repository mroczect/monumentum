use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

#[test]
fn and_all_true() {
    let result = unwrap_result(call_function(
        "AND",
        &[Value::Boolean(true), Value::Boolean(true)],
    ));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn and_has_false() {
    let result = unwrap_result(call_function(
        "AND",
        &[Value::Boolean(true), Value::Boolean(false)],
    ));
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn and_empty_errors() {
    assert!(matches!(
        call_function("AND", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn and_non_boolean_errors() {
    assert!(matches!(
        call_function("AND", &[Value::Integer(1.into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
