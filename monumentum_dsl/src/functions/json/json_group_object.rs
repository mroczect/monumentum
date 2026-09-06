#![allow(clippy::all)]
use alloc::boxed::Box;
use alloc::vec::Vec;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::json::JsonValue;
use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct JsonGroupObjectFunction;

impl AggregateFunction for JsonGroupObjectFunction {
    fn name(&self) -> &'static str {
        "json_group_object"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(JsonGroupObjectAccumulator { pairs: Vec::new() })
    }
}

#[derive(Debug)]
struct JsonGroupObjectAccumulator {
    pairs: Vec<(String, JsonValue)>,
}

impl Accumulator for JsonGroupObjectAccumulator {
    fn update(&mut self, args: &[Value]) -> Result<(), DbError> {
        if args.len() != 2 {
            return Err(DbError::invalid_operation(
                "json_group_object requires exactly two arguments",
            ));
        }
        let label = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("label must be text"))?;
        if label.is_null() {
            return Ok(());
        }
        let value = json_from_value(&args[1])?;
        self.pairs.push((label.to_string(), value));
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        let obj = JsonValue::Object(self.pairs);
        Value::try_from(obj.to_canonical_string())
    }
}

fn json_from_value(value: &Value) -> Result<JsonValue, DbError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Int(i.as_i64())),
        Value::Float(f) => Ok(JsonValue::Float(f.as_f64())),
        Value::Text(t) => Ok(JsonValue::Str(t.as_str().to_string())),
        Value::Blob(_) => Err(DbError::type_mismatch("BLOB not allowed")),
    }
}
