#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::parse_json;

#[derive(Debug, Clone, Copy)]
pub struct JsonValidFunction;

impl ScalarFunction for JsonValidFunction {
    fn name(&self) -> &'static str {
        "json_valid"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("json_valid requires one argument"))?;
        if arg.is_null() {
            return Ok(Value::Null);
        }
        let text = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("argument must be text"))?;
        let valid = parse_json(text).is_ok();
        Ok(Value::Boolean(valid))
    }
}
