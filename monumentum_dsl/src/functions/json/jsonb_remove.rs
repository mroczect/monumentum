
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::json_remove::JsonRemoveFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbRemoveFunction;

impl ScalarFunction for JsonbRemoveFunction {
    fn name(&self) -> &'static str {
        "jsonb_remove"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        JsonRemoveFunction.call(args)
    }
}