#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, extract_path, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonArrayInsertFunction;

impl ScalarFunction for JsonArrayInsertFunction {
    fn name(&self) -> &'static str {
        "json_array_insert"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() < 3 || args.len() % 2 == 0 {
            return Err(DbError::invalid_operation(
                "json_array_insert requires odd number of arguments (json, path, value, ...)",
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
            root = insert_into_array(root, path, value)?;
            i += 2;
        }
        Value::try_from(root.to_canonical_string())
    }
}

fn insert_into_array(root: JsonValue, path: &str, value: JsonValue) -> Result<JsonValue, DbError> {
    if let JsonValue::Array(mut arr) = root {
        if path == "$[#]" {
            arr.push(value);
            return Ok(JsonValue::Array(arr));
        } else if let Some(idx_str) = path.strip_prefix("$[") {
            if let Some(idx_end) = idx_str.find(']') {
                let idx: usize = idx_str[..idx_end]
                    .parse()
                    .map_err(|_| DbError::invalid_operation("invalid array index"))?;
                if idx <= arr.len() {
                    arr.insert(idx, value);
                    return Ok(JsonValue::Array(arr));
                }
            }
        }
    }
    Err(DbError::unsupported(
        "json_array_insert only supports root array paths",
    ))
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
