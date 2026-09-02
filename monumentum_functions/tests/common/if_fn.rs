use super::{call_function, unwrap_result};
use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

#[test]
fn if_true_returns_then() {
    let result = unwrap_result(call_function(
        "IF",
        &[
            Value::Boolean(true),
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(2)),
        ],
    ));
    assert_eq!(result, Value::Integer(Integer::new(1)));
}

#[test]
fn if_false_returns_else() {
    let result = unwrap_result(call_function(
        "IF",
        &[
            Value::Boolean(false),
            Value::Integer(Integer::new(1)),
            Value::Integer(Integer::new(2)),
        ],
    ));
    assert_eq!(result, Value::Integer(Integer::new(2)));
}

#[test]
fn if_wrong_arity_errors() {
    assert!(matches!(
        call_function(
            "IF",
            &[Value::Boolean(true), Value::Integer(Integer::new(1))]
        ),
        Err(FormulaError::WrongArity(_))
    ));
}

#[test]
fn if_non_boolean_condition_errors() {
    assert!(matches!(
        call_function(
            "IF",
            &[
                Value::Integer(Integer::new(1)),
                Value::Integer(Integer::new(1)),
                Value::Integer(Integer::new(2))
            ]
        ),
        Err(FormulaError::TypeMismatch(_))
    ));
}
