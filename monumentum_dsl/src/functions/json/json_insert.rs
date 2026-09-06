#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, json_from_value, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonInsertFunction;

impl ScalarFunction for JsonInsertFunction {
    fn name(&self) -> &'static str {
        "json_insert"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() < 3 || args.len() % 2 == 0 {
            return Err(DbError::invalid_operation(
                "json_insert requires odd number of arguments",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("first argument must be text"))?;
        let mut root = parse_json(json_text)?;
        let mut i = 1;
        while i < args.len() {
            let path = args[i]
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
            let value = json_from_value(&args[i + 1])?;
            root = insert_path(root, path, value)?;
            i += 2;
        }
        Value::try_from(root.to_canonical_string())
    }
}

fn insert_path(root: JsonValue, path: &str, value: JsonValue) -> Result<JsonValue, DbError> {
    if path == "$" {
        return Ok(value);
    }
    if let Some(key) = path.strip_prefix("$.") {
        let mut obj = match root {
            JsonValue::Object(o) => o,
            _ => Vec::new(),
        };
        if !obj.iter().any(|(k, _)| *k == key) {
            obj.push((key.to_string(), value));
        }
        Ok(JsonValue::Object(obj))
    } else if let Some(idx_str) = path.strip_prefix("$[") {
        if let Some(idx_end) = idx_str.find(']') {
            let idx: usize = idx_str[..idx_end]
                .parse()
                .map_err(|_| DbError::invalid_operation("invalid index"))?;
            let mut arr = match root {
                JsonValue::Array(a) => a,
                _ => Vec::new(),
            };
            if idx == arr.len() {
                arr.push(value);
            } else if idx > arr.len() {
                return Err(DbError::invalid_operation("index out of bounds"));
            }
            Ok(JsonValue::Array(arr))
        } else {
            Ok(root)
        }
    } else {
        Ok(root)
    }
}
