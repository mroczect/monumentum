use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct TotalFunction;

impl AggregateFunction for TotalFunction {
    fn name(&self) -> &'static str {
        "total"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(TotalAccumulator {
            sum: 0.0_f64,
            has_value: false,
        })
    }
}

#[derive(Debug)]
struct TotalAccumulator {
    sum: f64,
    has_value: bool,
}

impl Accumulator for TotalAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        match value {
            Value::Integer(i) => {
                #[allow(clippy::cast_precision_loss)]
                {
                    self.sum += i.as_i64() as f64;
                }
                self.has_value = true;
                Ok(())
            }
            Value::Float(f) => {
                self.sum += f.as_f64();
                self.has_value = true;
                Ok(())
            }
            Value::Null | Value::Text(_) | Value::Blob(_) | Value::Boolean(_) => Ok(()),
            _ => Ok(()),
        }
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        if self.has_value {
            Value::try_from(self.sum)
        } else {
            Value::try_from(0.0_f64)
        }
    }
}
