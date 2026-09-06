#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonReplaceFunction;

impl ScalarFunction for JsonReplaceFunction {
    fn name(&self) -> &'static str {
        "json_replace"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() < 3 || args.len() % 2 == 0 {
            return Err(DbError::invalid_operation(
                "json_replace requires odd number of arguments",
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
            root = replace_path(root, path, value)?;
            i += 2;
        }
        Value::try_from(root.to_canonical_string())
    }
}

fn replace_path(root: JsonValue, path: &str, value: JsonValue) -> Result<JsonValue, DbError> {
    if path == "$" {
        return Ok(value);
    }
    if let Some(key) = path.strip_prefix("$.") {
        if let JsonValue::Object(mut obj) = root {
            if let Some((_, v)) = obj.iter_mut().find(|(k, _)| *k == key) {
                *v = value;
                return Ok(JsonValue::Object(obj));
            }
        }
    } else if let Some(idx_str) = path.strip_prefix("$[") {
        if let Some(idx_end) = idx_str.find(']') {
            let idx: usize = idx_str[..idx_end]
                .parse()
                .map_err(|_| DbError::invalid_operation("invalid index"))?;
            if let JsonValue::Array(mut arr) = root {
                if let Some(elem) = arr.get_mut(idx) {
                    *elem = value;
                    return Ok(JsonValue::Array(arr));
                }
            }
        }
    }
    Ok(root)
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
