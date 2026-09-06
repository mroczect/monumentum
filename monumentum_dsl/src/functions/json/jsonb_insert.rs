
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::json_insert::JsonInsertFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbInsertFunction;

impl ScalarFunction for JsonbInsertFunction {
    fn name(&self) -> &'static str {
        "jsonb_insert"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        JsonInsertFunction.call(args)
    }
}