use super::{call_function, float_value, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::{Integer, Text};

#[test]
fn concat_text_and_numbers() {
    let result = unwrap_result(call_function(
        "CONCAT",
        &[
            Value::Text(Text::new("Hello".to_string())),
            Value::Text(Text::new(" ".to_string())),
            Value::Integer(Integer::new(2024)),
        ],
    ));
    assert_eq!(result, Value::Text(Text::new("Hello 2024".to_string())));
}

#[test]
fn concat_empty_args_returns_empty_string() {
    let result = unwrap_result(call_function("CONCAT", &[]));
    assert_eq!(result, Value::Text(Text::new(String::new())));
}

#[test]
fn concat_boolean_and_float() {
    let result = unwrap_result(call_function(
        "CONCAT",
        &[Value::Boolean(true), float_value(2.5)],
    ));
    assert_eq!(result, Value::Text(Text::new("true2.5".to_string())));
}
