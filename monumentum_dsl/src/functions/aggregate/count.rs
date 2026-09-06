use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct CountFunction;

impl AggregateFunction for CountFunction {
    fn name(&self) -> &'static str {
        "count"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(CountAccumulator { count: 0_u64 })
    }
}

#[derive(Debug)]
struct CountAccumulator {
    count: u64,
}

impl Accumulator for CountAccumulator {
    fn update(&mut self, _value: &Value) -> Result<(), DbError> {
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("count overflow"))?;
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        let count = i64::try_from(self.count)
            .map_err(|e| DbError::invalid_operation(format!("count too large for i64: {e}")))?;
        Ok(Value::from(count))
    }
}
