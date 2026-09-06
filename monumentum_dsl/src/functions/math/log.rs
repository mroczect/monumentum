use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::math::value_to_f64;

#[derive(Debug, Clone, Copy)]
pub struct LogFunction;

impl crate::functions::ScalarFunction for LogFunction {
    fn name(&self) -> &'static str {
        "log"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        match args.len() {
            1 => {
                let Some(x) = args.first().and_then(value_to_f64) else {
                    return Ok(Value::Null);
                };
                let result = x.log10();
                if result.is_finite() {
                    Value::try_from(result)
                } else {
                    Ok(Value::Null)
                }
            }
            2 => {
                let Some(b) = args.first().and_then(value_to_f64) else {
                    return Ok(Value::Null);
                };
                let Some(x) = args.get(1).and_then(value_to_f64) else {
                    return Ok(Value::Null);
                };
                if b <= 0.0 || (b - 1.0).abs() < f64::EPSILON || x <= 0.0 {
                    return Ok(Value::Null);
                }
                let result = x.log(b);
                if result.is_finite() {
                    Value::try_from(result)
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Ok(Value::Null),
        }
    }
}
