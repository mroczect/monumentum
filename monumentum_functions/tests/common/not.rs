use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

#[test]
fn not_true_becomes_false() {
    let result = unwrap_result(call_function("NOT", &[Value::Boolean(true)]));
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn not_false_becomes_true() {
    let result = unwrap_result(call_function("NOT", &[Value::Boolean(false)]));
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn not_wrong_arity_errors() {
    assert!(matches!(
        call_function("NOT", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn not_non_boolean_errors() {
    assert!(matches!(
        call_function("NOT", &[Value::Integer(1.into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
