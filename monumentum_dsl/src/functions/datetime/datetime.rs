#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction,
    clippy::arithmetic_side_effects
)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::datetime::{apply_modifiers, format_datetime, parse_time_value, split_args};

#[derive(Debug, Clone, Copy)]
pub struct DatetimeFunction;

impl ScalarFunction for DatetimeFunction {
    fn name(&self) -> &'static str {
        "datetime"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let (value, modifiers) = split_args(args)?;
        let mut tp = parse_time_value(&value, &modifiers)?;
        tp = apply_modifiers(tp, &modifiers)?;
        let subsec = modifiers.iter().any(|m| m == "subsec" || m == "subsecond");
        let result = format_datetime(&tp, subsec);
        Value::try_from(result)
    }
}
