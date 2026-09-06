#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::JsonValue;

#[derive(Debug, Clone, Copy)]
pub struct JsonObjectFunction;

impl ScalarFunction for JsonObjectFunction {
    fn name(&self) -> &'static str {
        "json_object"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() % 2 != 0 {
            return Err(DbError::invalid_operation(
                "json_object requires even number of arguments",
            ));
        }
        let mut obj = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let label = args[i]
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("label must be text"))?;
            let value = &args[i + 1];
            let jv = match value {
                Value::Null => JsonValue::Null,
                Value::Boolean(b) => JsonValue::Bool(*b),
                Value::Integer(ii) => JsonValue::Int(ii.as_i64()),
                Value::Float(f) => JsonValue::Float(f.as_f64()),
                Value::Text(t) => JsonValue::Str(t.as_str().to_string()),
                Value::Blob(_) => return Err(DbError::type_mismatch("BLOB not allowed")),
            };
            obj.push((label.to_string(), jv));
            i += 2;
        }
        Value::try_from(JsonValue::Object(obj).to_canonical_string())
    }
}
