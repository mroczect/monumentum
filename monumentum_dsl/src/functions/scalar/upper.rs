use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct UpperFunction;

impl ScalarFunction for UpperFunction {
    fn name(&self) -> &'static str {
        "upper"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("upper expects one argument"))?;
        let s = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("upper expects text"))?;
        Value::try_from(s.to_uppercase())
    }
}
