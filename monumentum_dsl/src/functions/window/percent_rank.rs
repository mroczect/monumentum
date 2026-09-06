use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct PercentRankFunction;

impl WindowFunction for PercentRankFunction {
    fn name(&self) -> &'static str {
        "percent_rank"
    }

    fn evaluate(
        &self,
        partition: &[Row],
        current_idx: usize,
        _args: &[Value],
        order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        let n = partition.len();
        if n <= 1 {
            return Value::try_from(0.0_f64);
        }

        let rank = if order_values.is_empty() {
            1_i64
        } else {
            let current_order = order_values
                .get(current_idx)
                .ok_or_else(|| DbError::invalid_operation("current index out of bounds"))?;
            let mut rank_val = 1_i64;
            for (idx, ov) in order_values.iter().enumerate() {
                if idx >= current_idx {
                    break;
                }
                if ov != current_order {
                    rank_val = rank_val
                        .checked_add(1)
                        .ok_or_else(|| DbError::invalid_operation("rank overflow"))?;
                }
            }
            rank_val
        };

        let numerator = rank
            .checked_sub(1)
            .ok_or_else(|| DbError::invalid_operation("rank underflow"))?;
        let denominator = n
            .checked_sub(1)
            .ok_or_else(|| DbError::invalid_operation("partition size underflow"))?;

        #[allow(clippy::cast_precision_loss)]
        let percent = numerator as f64 / denominator as f64;
        Value::try_from(percent)
    }
}
