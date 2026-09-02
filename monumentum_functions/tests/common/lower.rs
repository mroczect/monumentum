use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Text;
use monumentum_query::formula::FormulaError;

#[test]
fn lower_converts_to_lowercase() {
    let result = unwrap_result(call_function(
        "LOWER",
        &[Value::Text(Text::new("HELLO".to_string()))],
    ));
    assert_eq!(result, Value::Text(Text::new("hello".to_string())));
}

#[test]
fn lower_wrong_arity_errors() {
    assert!(matches!(
        call_function("LOWER", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn lower_non_text_errors() {
    assert!(matches!(
        call_function("LOWER", &[Value::Integer(1.into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
