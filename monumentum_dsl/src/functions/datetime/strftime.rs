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
use crate::functions::datetime::{apply_modifiers, parse_time_value, split_args, strftime};

#[derive(Debug, Clone, Copy)]
pub struct StrftimeFunction;

impl ScalarFunction for StrftimeFunction {
    fn name(&self) -> &'static str {
        "strftime"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "strftime requires format string",
            ));
        }
        let format = match args.first() {
            Some(Value::Text(s)) => s.as_str().to_string(),
            _ => return Err(DbError::type_mismatch("format must be text")),
        };
        let (value, modifiers) = if args.len() >= 2 {
            split_args(&args[1..])?
        } else {
            split_args(&[])?
        };
        let mut tp = parse_time_value(&value, &modifiers)?;
        tp = apply_modifiers(tp, &modifiers)?;
        match strftime(&format, &tp) {
            Some(s) => Value::try_from(s),
            None => Ok(Value::Null),
        }
    }
}
