#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{JsonValue, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonPatchFunction;

impl ScalarFunction for JsonPatchFunction {
    fn name(&self) -> &'static str {
        "json_patch"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Err(DbError::invalid_operation(
                "json_patch requires exactly two arguments",
            ));
        }
        let target_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("target must be text"))?;
        let patch_text = args[1]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("patch must be text"))?;
        let target = parse_json(target_text)?;
        let patch = parse_json(patch_text)?;
        let result = merge_patch(target, patch)?;
        Value::try_from(result.to_canonical_string())
    }
}

fn merge_patch(target: JsonValue, patch: JsonValue) -> Result<JsonValue, DbError> {
    match patch {
        JsonValue::Object(patch_obj) => {
            let mut target_obj = match target {
                JsonValue::Object(o) => o,
                _ => Vec::new(),
            };
            for (key, patch_val) in patch_obj {
                match patch_val {
                    JsonValue::Null => {
                        target_obj.retain(|(k, _)| *k != key);
                    }
                    _ => {
                        let existing = target_obj.iter_mut().find(|(k, _)| *k == key);
                        if let Some((_, val)) = existing {
                            *val = merge_patch(val.clone(), patch_val)?;
                        } else {
                            target_obj.push((key, patch_val));
                        }
                    }
                }
            }
            Ok(JsonValue::Object(target_obj))
        }
        _ => Ok(patch),
    }
}
