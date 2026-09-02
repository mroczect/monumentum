use monumentum_db::core::value::Value;
use monumentum_db::types::Text;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    match args {
        [single] => match single {
            Value::Text(t) => Ok(Value::Text(Text::new(t.as_str().to_lowercase()))),
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => Err(FormulaError::TypeMismatch("LOWER expects text".to_string())),
        },
        _ => Err(FormulaError::WrongArity(
            "LOWER expects 1 argument".to_string(),
        )),
    }
}
