use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct AvgFunction;

impl AggregateFunction for AvgFunction {
    fn name(&self) -> &'static str {
        "avg"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(AvgAccumulator {
            sum: 0_i64,
            count: 0_u64,
        })
    }
}

#[derive(Debug)]
struct AvgAccumulator {
    sum: i64,
    count: u64,
}

impl Accumulator for AvgAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        if let Some(i) = value.as_i64() {
            self.sum = self
                .sum
                .checked_add(i)
                .ok_or_else(|| DbError::invalid_operation("sum overflow"))?;
            self.count = self
                .count
                .checked_add(1)
                .ok_or_else(|| DbError::invalid_operation("count overflow"))?;
        } else {
            return Err(DbError::type_mismatch("avg expects integer"));
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        if self.count == 0 {
            return Ok(Value::Null);
        }
        #[allow(clippy::cast_precision_loss)]
        let avg = self.sum as f64 / self.count as f64;
        Value::try_from(avg)
    }
}
