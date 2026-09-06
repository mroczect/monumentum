
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::json_patch::JsonPatchFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbPatchFunction;

impl ScalarFunction for JsonbPatchFunction {
    fn name(&self) -> &'static str {
        "jsonb_patch"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        JsonPatchFunction.call(args)
    }
}