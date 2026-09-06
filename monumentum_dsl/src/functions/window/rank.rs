use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct RankFunction;

impl WindowFunction for RankFunction {
    fn name(&self) -> &'static str {
        "rank"
    }

    fn evaluate(
        &self,
        _partition: &[Row],
        current_idx: usize,
        _args: &[Value],
        order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        if order_values.is_empty() {
            return Ok(Value::from(1_i64));
        }

        let current_order = order_values
            .get(current_idx)
            .ok_or_else(|| DbError::invalid_operation("current index out of bounds"))?;
        let mut rank = 1_i64;
        for (idx, ov) in order_values.iter().enumerate() {
            if idx >= current_idx {
                break;
            }
            if ov != current_order {
                rank = rank
                    .checked_add(1)
                    .ok_or_else(|| DbError::invalid_operation("rank overflow"))?;
            }
        }
        Ok(Value::from(rank))
    }
}
