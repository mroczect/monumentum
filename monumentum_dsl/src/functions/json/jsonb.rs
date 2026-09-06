#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::parse_json;

#[derive(Debug, Clone, Copy)]
pub struct JsonbFunction;

impl ScalarFunction for JsonbFunction {
    fn name(&self) -> &'static str {
        "jsonb"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("jsonb requires one argument"))?;
        let text = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("jsonb argument must be text"))?;
        let parsed = parse_json(text)?;
        Value::try_from(parsed.to_canonical_string())
    }
}
