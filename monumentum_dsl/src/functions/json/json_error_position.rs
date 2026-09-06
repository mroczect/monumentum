#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::parse_json;

#[derive(Debug, Clone, Copy)]
pub struct JsonErrorPositionFunction;

impl ScalarFunction for JsonErrorPositionFunction {
    fn name(&self) -> &'static str {
        "json_error_position"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args.first().ok_or_else(|| {
            DbError::invalid_operation("json_error_position requires one argument")
        })?;
        let text = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("argument must be text"))?;
        match parse_json(text) {
            Ok(_) => Ok(Value::from(0_i64)),
            Err(_) => Ok(Value::from(1_i64)),
        }
    }
}
