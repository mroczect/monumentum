use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

// ============ Group Concat ============

#[derive(Debug, Clone)]
pub struct GroupConcatFunction {
    separator: String,
}

impl GroupConcatFunction {
    #[must_use]
    pub fn new(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
        }
    }
}

impl Default for GroupConcatFunction {
    fn default() -> Self {
        Self::new(",")
    }
}

impl AggregateFunction for GroupConcatFunction {
    fn name(&self) -> &'static str {
        "group_concat"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(GroupConcatAccumulator {
            separator: self.separator.clone(),
            values: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct GroupConcatAccumulator {
    separator: String,
    values: Vec<String>,
}

impl Accumulator for GroupConcatAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        if let Some(s) = value.as_str() {
            self.values.push(s.to_string());
            Ok(())
        } else {
            Err(DbError::type_mismatch("group_concat expects text values"))
        }
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        let joined = self.values.join(&self.separator);
        Value::try_from(joined)
    }
}

#[derive(Debug, Clone)]
pub struct StringAggFunction {
    separator: String,
}

impl StringAggFunction {
    #[must_use]
    pub fn new(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
        }
    }
}

impl Default for StringAggFunction {
    fn default() -> Self {
        Self::new(",")
    }
}

impl AggregateFunction for StringAggFunction {
    fn name(&self) -> &'static str {
        "string_agg"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(GroupConcatAccumulator {
            separator: self.separator.clone(),
            values: Vec::new(),
        })
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct MedianFunction;

impl AggregateFunction for MedianFunction {
    fn name(&self) -> &'static str {
        "median"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(PercentileContAccumulator {
            p: 0.5_f64,
            values: Vec::new(),
        })
    }
}

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
        Box::new(PercentileContAccumulator {
            p: self.p,
            values: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct PercentileContAccumulator {
    p: f64,
    values: Vec<f64>,
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

#[derive(Debug, Clone, Copy)]
pub struct PercentileDiscFunction {
    p: f64,
}

impl PercentileDiscFunction {
    #[must_use]
    pub const fn new(p: f64) -> Self {
        Self { p }
    }
}

impl AggregateFunction for PercentileDiscFunction {
    fn name(&self) -> &'static str {
        "percentile_disc"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(PercentileDiscAccumulator {
            p: self.p,
            values: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct PercentileDiscAccumulator {
    p: f64,
    values: Vec<f64>,
}

impl Accumulator for PercentileDiscAccumulator {
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
        let idx = ((n as f64 * self.p).ceil() as usize).saturating_sub(1);
        let result = self
            .values
            .get(idx.min(n.saturating_sub(1)))
            .copied()
            .unwrap_or(0.0_f64);
        Value::try_from(result)
    }
}
