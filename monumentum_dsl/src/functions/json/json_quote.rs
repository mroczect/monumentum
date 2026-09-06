#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::JsonValue;

#[derive(Debug, Clone, Copy)]
pub struct JsonQuoteFunction;

impl ScalarFunction for JsonQuoteFunction {
    fn name(&self) -> &'static str {
        "json_quote"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("json_quote requires one argument"))?;
        match arg {
            Value::Null => Ok(Value::try_from("null".to_string())?),
            Value::Boolean(b) => Ok(Value::try_from(b.to_string())?),
            Value::Integer(i) => Ok(Value::try_from(i.as_i64().to_string())?),
            Value::Float(f) => Ok(Value::try_from(f.as_f64().to_string())?),
            Value::Text(t) => {
                let s = t.as_str();
                let quoted = JsonValue::Str(s.to_string()).to_canonical_string();
                Value::try_from(quoted)
            }
            Value::Blob(_) => Err(DbError::type_mismatch("BLOB not supported")),
        }
    }
}
