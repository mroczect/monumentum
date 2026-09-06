#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{extract_path, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonExtractFunction;

impl ScalarFunction for JsonExtractFunction {
    fn name(&self) -> &'static str {
        "json_extract"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() < 2 {
            return Err(DbError::invalid_operation(
                "json_extract requires at least 2 arguments",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("first argument must be text"))?;
        let root = parse_json(json_text)?;

        if args.len() == 2 {
            let path = args[1]
                .as_str()
                .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
            let result = extract_path(&root, path)?;
            match result {
                Some(jv) => jv.to_sql_value(),
                None => Ok(Value::Null),
            }
        } else {
            let mut arr = Vec::new();
            for arg in &args[1..] {
                let path = arg
                    .as_str()
                    .ok_or_else(|| DbError::type_mismatch("path must be text"))?;
                let r = extract_path(&root, path)?;
                let jv = r.unwrap_or(JsonValue::Null);
                arr.push(jv);
            }
            Value::try_from(JsonValue::Array(arr).to_canonical_string())
        }
    }
}
