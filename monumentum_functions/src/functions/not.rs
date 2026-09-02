use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [single] => match single {
            Value::Boolean(b) => Ok(Value::Boolean(!*b)),
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Formula(_)
            | _ => Err(FormulaError::TypeMismatch(
                "NOT requires boolean argument".to_string(),
            )),
        },
        _ => Err(FormulaError::WrongArity(
            "NOT expects 1 argument".to_string(),
        )),
    }
}
