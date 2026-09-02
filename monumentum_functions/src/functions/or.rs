use monumentum_db::core::value::Value;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::WrongArity(
            "OR expects at least one argument".to_string(),
        ));
    }

    for arg in args {
        match arg {
            Value::Boolean(true) => return Ok(Value::Boolean(true)),
            Value::Boolean(false) => {}
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Formula(_)
            | _ => {
                return Err(FormulaError::TypeMismatch(
                    "OR requires boolean arguments".to_string(),
                ));
            }
        }
    }
    Ok(Value::Boolean(false))
}
