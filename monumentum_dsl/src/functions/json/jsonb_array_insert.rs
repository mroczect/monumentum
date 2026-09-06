#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonbArrayInsertFunction;

impl ScalarFunction for JsonbArrayInsertFunction {
    fn name(&self) -> &'static str {
        "jsonb_array_insert"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        super::json_array_insert::JsonArrayInsertFunction.call(args)
    }
}

fn json_from_value(value: &Value) -> Result<JsonValue, DbError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Int(i.as_i64())),
        Value::Float(f) => Ok(JsonValue::Float(f.as_f64())),
        Value::Text(t) => Ok(JsonValue::Str(t.as_str().to_string())),
        Value::Blob(_) => Err(DbError::type_mismatch("BLOB not allowed")),
    }
}
