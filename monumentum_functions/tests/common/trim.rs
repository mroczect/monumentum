use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Text;
use monumentum_query::formula::FormulaError;

#[test]
fn trim_removes_whitespace() {
    let result = unwrap_result(call_function(
        "TRIM",
        &[Value::Text(Text::new("  hello  ".to_string()))],
    ));
    assert_eq!(result, Value::Text(Text::new("hello".to_string())));
}

#[test]
fn trim_wrong_arity_errors() {
    assert!(matches!(
        call_function("TRIM", &[]),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn trim_non_text_errors() {
    assert!(matches!(
        call_function("TRIM", &[Value::Integer(1.into())]),
        Err(FormulaError::TypeMismatch(_))
    ));
}
