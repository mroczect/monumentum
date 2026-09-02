use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::WrongArity(
            "AND expects at least one argument".to_string(),
        ));
    }

    for arg in args {
        match arg {
            Value::Boolean(true) => {}
            Value::Boolean(false) => return Ok(Value::Boolean(false)),
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Formula(_)
            | _ => {
                return Err(FormulaError::TypeMismatch(
                    "AND requires boolean arguments".to_string(),
                ));
            }
        }
    }
    Ok(Value::Boolean(true))
}
