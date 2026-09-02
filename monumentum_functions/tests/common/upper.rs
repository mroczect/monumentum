use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Text;
use monumentum_query::formula::FormulaError;

#[test]
fn upper_converts_to_uppercase() {
    let result = unwrap_result(call_function(
        "UPPER",
        &[Value::Text(Text::new("hello".to_string()))],
    ));
    assert_eq!(result, Value::Text(Text::new("HELLO".to_string())));
}

#[test]
fn upper_wrong_arity_errors() {
    assert!(matches!(
        call_function("UPPER", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn upper_non_text_errors() {
    assert!(matches!(
        call_function("UPPER", &[Value::Boolean(true)]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
