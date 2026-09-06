use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::math::value_to_f64;

#[derive(Debug, Clone, Copy)]
pub struct Atan2Function;

impl crate::functions::ScalarFunction for Atan2Function {
    fn name(&self) -> &'static str {
        "atan2"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let Some(y) = args.first().and_then(value_to_f64) else {
            return Ok(Value::Null);
        };
        let Some(x) = args.get(1).and_then(value_to_f64) else {
            return Ok(Value::Null);
        };
        let result = y.atan2(x);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}
