#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonTreeFunction;

impl ScalarFunction for JsonTreeFunction {
    fn name(&self) -> &'static str {
        "json_tree"
    }

    fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
        Err(DbError::unsupported(
            "json_tree is a table-valued function and cannot be used as scalar",
        ))
    }
}
