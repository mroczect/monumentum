use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct LowerFunction;

impl ScalarFunction for LowerFunction {
    fn name(&self) -> &'static str {
        "lower"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("lower expects one argument"))?;
        let s = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("lower expects text"))?;
        Value::try_from(s.to_lowercase())
    }
}
