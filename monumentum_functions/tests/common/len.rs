use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::{Integer, Text};
use monumentum_query::formula::FormulaError;

#[test]
fn len_returns_text_length() {
    let result = unwrap_result(call_function(
        "LEN",
        &[Value::Text(Text::new("hello".to_string()))],
    ));
    assert_eq!(result, Value::Integer(Integer::new(5)));
}

#[test]
fn len_wrong_arity_errors() {
    assert!(matches!(
        call_function("LEN", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn len_non_text_errors() {
    assert!(matches!(
        call_function("LEN", &[Value::Boolean(false)]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
