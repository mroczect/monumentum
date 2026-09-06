#![allow(clippy::all)]
use alloc::boxed::Box;
use alloc::vec::Vec;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::json::JsonValue;
use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct JsonGroupArrayFunction;

impl AggregateFunction for JsonGroupArrayFunction {
    fn name(&self) -> &'static str {
        "json_group_array"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(JsonGroupArrayAccumulator { items: Vec::new() })
    }
}

#[derive(Debug)]
struct JsonGroupArrayAccumulator {
    items: Vec<JsonValue>,
}

impl Accumulator for JsonGroupArrayAccumulator {
    fn update(&mut self, args: &[Value]) -> Result<(), DbError> {
        if args.len() != 1 {
            return Err(DbError::invalid_operation(
                "json_group_array requires exactly one argument",
            ));
        }
        let value = &args[0];
        let jv = match value {
            Value::Null => JsonValue::Null,
            Value::Boolean(b) => JsonValue::Bool(*b),
            Value::Integer(i) => JsonValue::Int(i.as_i64()),
            Value::Float(f) => JsonValue::Float(f.as_f64()),
            Value::Text(t) => JsonValue::Str(t.as_str().to_string()),
            Value::Blob(_) => return Err(DbError::type_mismatch("BLOB not allowed")),
        };
        self.items.push(jv);
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        let arr = JsonValue::Array(self.items);
        Value::try_from(arr.to_canonical_string())
    }
}
