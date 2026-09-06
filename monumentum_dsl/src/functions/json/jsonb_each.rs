
#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct JsonbEachFunction;

impl ScalarFunction for JsonbEachFunction {
    fn name(&self) -> &'static str {
        "jsonb_each"
    }

    fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
        Err(DbError::unsupported(
            "jsonb_each is a table-valued function and cannot be used as scalar",
        ))
    }
}