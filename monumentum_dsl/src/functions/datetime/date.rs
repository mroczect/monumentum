#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::datetime::{apply_modifiers, format_date, parse_time_value, split_args};

#[derive(Debug, Clone, Copy)]
pub struct DateFunction;

impl ScalarFunction for DateFunction {
    fn name(&self) -> &'static str {
        "date"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let (value, modifiers) = split_args(args)?;
        let mut tp = parse_time_value(&value, &modifiers)?;
        tp = apply_modifiers(tp, &modifiers)?;
        let result = format_date(&tp);
        Value::try_from(result)
    }
}
