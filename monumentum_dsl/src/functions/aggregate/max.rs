use alloc::boxed::Box;
use core::cmp::Ordering;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct MaxFunction;

impl AggregateFunction for MaxFunction {
    fn name(&self) -> &'static str {
        "max"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(MinMaxAccumulator {
            best: None,
            is_min: false,
        })
    }
}

#[derive(Debug)]
struct MinMaxAccumulator {
    best: Option<Value>,
    is_min: bool,
}

impl Accumulator for MinMaxAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        let should_replace = match &self.best {
            None => true,
            Some(best) => {
                let cmp = value
                    .partial_cmp(best)
                    .ok_or_else(|| DbError::type_mismatch("cannot compare values for min/max"))?;
                if self.is_min {
                    cmp == Ordering::Less
                } else {
                    cmp == Ordering::Greater
                }
            }
        };
        if should_replace {
            self.best = Some(value.clone());
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        Ok(self.best.unwrap_or(Value::Null))
    }
}
