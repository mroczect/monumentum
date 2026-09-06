#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, extract_path, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonTypeFunction;

impl ScalarFunction for JsonTypeFunction {
    fn name(&self) -> &'static str {
        "json_type"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "json_type requires at least one argument",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("first argument must be text"))?;
        let root = parse_json(json_text)?;
        let value = if args.len() >= 2 {
            let path = args[1]
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
            extract_path(&root, path)?
        } else {
            Some(root)
        };
        let type_str = match value {
            Some(JsonValue::Null) => "null",
            Some(JsonValue::Bool(_)) => "boolean",
            Some(JsonValue::Int(_)) => "integer",
            Some(JsonValue::Float(_)) => "real",
            Some(JsonValue::Str(_)) => "text",
            Some(JsonValue::Array(_)) => "array",
            Some(JsonValue::Object(_)) => "object",
            None => return Ok(Value::Null),
        };
        Value::try_from(type_str.to_string())
    }
}
