use monumentum_db::core::value::Value;
use monumentum_db::types::Integer;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [single] => match single {
            Value::Text(t) => {
                let len = i64::try_from(t.len())
                    .map_err(|e| FormulaError::Eval(format!("text length too large: {e}")))?;
                Ok(Value::Integer(Integer::new(len)))
            }
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => Err(FormulaError::TypeMismatch("LEN expects text".to_string())),
        },
        _ => Err(FormulaError::WrongArity(
            "LEN expects 1 argument".to_string(),
        )),
    }
}
