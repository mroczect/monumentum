#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, extract_path, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonArrayLengthFunction;

impl ScalarFunction for JsonArrayLengthFunction {
    fn name(&self) -> &'static str {
        "json_array_length"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "json_array_length requires at least one argument",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("first argument must be text"))?;
        let root = parse_json(json_text)?;
        let target = if args.len() >= 2 {
            let path = args[1]
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
            extract_path(&root, path)?
        } else {
            Some(root)
        };
        let len = match target {
            Some(JsonValue::Array(arr)) => arr.len() as i64,
            _ => 0,
        };
        Ok(Value::from(len))
    }
}
