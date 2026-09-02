use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

#[test]
fn or_all_false() {
    let result = unwrap_result(call_function(
        "OR",
        &[Value::Boolean(false), Value::Boolean(false)],
    ));
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn or_has_true() {
    let result = unwrap_result(call_function(
        "OR",
        &[Value::Boolean(false), Value::Boolean(true)],
    ));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn or_empty_errors() {
    assert!(matches!(
        call_function("OR", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn or_non_boolean_errors() {
    assert!(matches!(
        call_function("OR", &[Value::Text("x".to_string().into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
