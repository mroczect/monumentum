#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbTreeFunction;

impl ScalarFunction for JsonbTreeFunction {
    fn name(&self) -> &'static str {
        "jsonb_tree"
    }

    fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
        Err(DbError::unsupported(
            "jsonb_tree is a table-valued function and cannot be used as scalar",
        ))
    }
}
