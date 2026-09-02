use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [cond, then_val, else_val] => match cond {
            Value::Boolean(true) => Ok(then_val.clone()),
            Value::Boolean(false) => Ok(else_val.clone()),
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Formula(_)
            | _ => Err(FormulaError::TypeMismatch(
                "IF condition must be boolean".to_string(),
            )),
        },
        _ => Err(FormulaError::WrongArity(
            "IF expects 3 arguments".to_string(),
        )),
    }
}
