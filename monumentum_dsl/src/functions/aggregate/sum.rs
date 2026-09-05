use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct SumFunction;

impl AggregateFunction for SumFunction {
    fn name(&self) -> &'static str {
        "sum"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(SumAccumulator {
            sum: 0_i64,
            has_value: false,
        })
    }
}

#[derive(Debug)]
struct SumAccumulator {
    sum: i64,
    has_value: bool,
}

impl Accumulator for SumAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        if let Some(i) = value.as_i64() {
            self.sum = self
                .sum
                .checked_add(i)
                .ok_or_else(|| DbError::invalid_operation("sum overflow"))?;
            self.has_value = true;
        } else {
            return Err(DbError::type_mismatch("sum expects integer"));
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        if self.has_value {
            Ok(Value::from(self.sum))
        } else {
            Ok(Value::Null)
        }
    }
}
