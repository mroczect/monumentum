use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::math::value_to_f64;

#[derive(Debug, Clone, Copy)]
pub struct ModFunction;

impl crate::functions::ScalarFunction for ModFunction {
    fn name(&self) -> &'static str {
        "mod"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let Some(x) = args.first().and_then(value_to_f64) else {
            return Ok(Value::Null);
        };
        let Some(y) = args.get(1).and_then(value_to_f64) else {
            return Ok(Value::Null);
        };
        if y == 0.0 {
            return Ok(Value::Null);
        }
        let result = (x / y).trunc().mul_add(-y, x);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}
