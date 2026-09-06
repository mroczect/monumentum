
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::json_set::JsonSetFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbSetFunction;

impl ScalarFunction for JsonbSetFunction {
    fn name(&self) -> &'static str {
        "jsonb_set"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        JsonSetFunction.call(args)
    }
}