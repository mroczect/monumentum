#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::datetime::{parse_time_value, to_julian_day};

#[derive(Debug, Clone, Copy)]
pub struct TimediffFunction;

impl ScalarFunction for TimediffFunction {
    fn name(&self) -> &'static str {
        "timediff"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Err(DbError::invalid_operation(
                "timediff requires exactly two arguments",
            ));
        }
        let a = parse_time_value(&args[0], &[])?;
        let b = parse_time_value(&args[1], &[])?;
        let diff_seconds = (to_julian_day(&a) - to_julian_day(&b)) * 86_400.0;
        let sign = if diff_seconds < 0.0 { "-" } else { "+" };
        let total = diff_seconds.abs();
        let days = (total / 86_400.0).floor() as i64;
        let rem = total - (days as f64) * 86_400.0;
        let hours = (rem / 3_600.0).floor() as i64;
        let rem = rem - (hours as f64) * 3_600.0;
        let minutes = (rem / 60.0).floor() as i64;
        let seconds = rem - (minutes as f64) * 60.0;
        let years = days / 365;
        let months = (days % 365) / 30;
        let days_left = days % 30;
        let result = format!(
            "{}{:04}-{:02}-{:02} {:02}:{:02}:{:06.3}",
            sign, years, months, days_left, hours, minutes, seconds
        );
        Value::try_from(result)
    }
}
