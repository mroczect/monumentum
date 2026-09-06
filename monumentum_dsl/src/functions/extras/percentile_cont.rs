use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct PercentileContFunction {
    p: f64,
}

impl PercentileContFunction {
    #[must_use]
    pub const fn new(p: f64) -> Self {
        Self { p }
    }
}

impl AggregateFunction for PercentileContFunction {
    fn name(&self) -> &'static str {
        "percentile_cont"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(PercentileContAccumulator::new(self.p))
    }
}

#[derive(Debug)]
pub(crate) struct PercentileContAccumulator {
    p: f64,
    values: Vec<f64>,
}

impl PercentileContAccumulator {
    pub(crate) const fn new(p: f64) -> Self {
        Self {
            p,
            values: Vec::new(),
        }
    }
}

impl Accumulator for PercentileContAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        match value {
            Value::Integer(i) => {
                #[allow(clippy::cast_precision_loss)]
                {
                    self.values.push(i.as_i64() as f64);
                }
                Ok(())
            }
            Value::Float(f) => {
                self.values.push(f.as_f64());
                Ok(())
            }
            Value::Null => Ok(()),
            Value::Text(_) | Value::Blob(_) | Value::Boolean(_) => {
                Err(DbError::type_mismatch("percentile expects numeric values"))
            }
            _ => Err(DbError::type_mismatch("percentile expects numeric values")),
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn finish(mut self: Box<Self>) -> Result<Value, DbError> {
        if self.values.is_empty() {
            return Ok(Value::Null);
        }
        if !(0.0..=1.0).contains(&self.p) {
            return Err(DbError::invalid_operation(
                "percentile p must be between 0 and 1",
            ));
        }
        self.values
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let n = self.values.len();
        let rank = (n as f64 - 1.0_f64) * self.p;
        let low = rank.floor() as usize;
        let high = rank.ceil() as usize;
        let frac = rank - low as f64;
        let low_val = self.values.get(low).copied().unwrap_or(0.0_f64);
        let high_val = self.values.get(high).copied().unwrap_or(0.0_f64);
        let result = if low == high {
            low_val
        } else {
            high_val.mul_add(frac, low_val * (1.0_f64 - frac))
        };
        Value::try_from(result)
    }
}
