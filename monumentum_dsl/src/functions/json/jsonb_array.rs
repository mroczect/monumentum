#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::JsonValue;

#[derive(Debug, Clone, Copy)]
pub struct JsonbArrayFunction;

impl ScalarFunction for JsonbArrayFunction {
    fn name(&self) -> &'static str {
        "jsonb_array"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let mut arr = Vec::new();
        for arg in args {
            let jv = match arg {
                Value::Null => JsonValue::Null,
                Value::Boolean(b) => JsonValue::Bool(*b),
                Value::Integer(i) => JsonValue::Int(i.as_i64()),
                Value::Float(f) => JsonValue::Float(f.as_f64()),
                Value::Text(t) => JsonValue::Str(t.as_str().to_string()),
                Value::Blob(_) => return Err(DbError::type_mismatch("BLOB not allowed")),
            };
            arr.push(jv);
        }
        Value::try_from(JsonValue::Array(arr).to_canonical_string())
    }
}
