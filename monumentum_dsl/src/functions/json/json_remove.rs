#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{parse_json, JsonValue};

#[derive(Debug, Clone, Copy)]
pub struct JsonRemoveFunction;

impl ScalarFunction for JsonRemoveFunction {
    fn name(&self) -> &'static str {
        "json_remove"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "json_remove requires at least one argument",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("first argument must be text"))?;
        let mut root = parse_json(json_text)?;
        for path_arg in &args[1..] {
            let path = path_arg
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
            root = remove_path(root, path)?;
        }
        Value::try_from(root.to_canonical_string())
    }
}

fn remove_path(root: JsonValue, path: &str) -> Result<JsonValue, DbError> {
    if path == "$" {
        return Ok(JsonValue::Null);
    }
    if let Some(key) = path.strip_prefix("$.") {
        if let JsonValue::Object(mut obj) = root {
            obj.retain(|(k, _)| k != key);
            return Ok(JsonValue::Object(obj));
        }
    } else if let Some(idx_str) = path.strip_prefix("$[") {
        if let Some(idx_end) = idx_str.find(']') {
            let idx: usize = idx_str[..idx_end]
                .parse()
                .map_err(|_| DbError::invalid_operation("invalid index"))?;
            if let JsonValue::Array(mut arr) = root {
                if idx < arr.len() {
                    arr.remove(idx);
                }
                return Ok(JsonValue::Array(arr));
            }
        }
    }
    Ok(root)
}
