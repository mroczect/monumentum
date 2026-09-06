use core::f64::consts::PI;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[derive(Debug, Clone, Copy)]
pub struct PiFunction;

impl crate::functions::ScalarFunction for PiFunction {
    fn name(&self) -> &'static str {
        "pi"
    }

    fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
        Value::try_from(PI)
    }
}
