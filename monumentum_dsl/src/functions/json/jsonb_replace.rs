
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::json_replace::JsonReplaceFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbReplaceFunction;

impl ScalarFunction for JsonbReplaceFunction {
    fn name(&self) -> &'static str {
        "jsonb_replace"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        JsonReplaceFunction.call(args)
    }
}