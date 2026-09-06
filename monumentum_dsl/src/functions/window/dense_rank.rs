use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct DenseRankFunction;

impl WindowFunction for DenseRankFunction {
    fn name(&self) -> &'static str {
        "dense_rank"
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
        let mut dense_rank = 1_i64;
        let mut last_order: Option<&Option<Value>> = None;
        for ov in order_values.iter().take(current_idx) {
            if last_order != Some(ov) {
                if ov != current_order {
                    dense_rank = dense_rank
                        .checked_add(1)
                        .ok_or_else(|| DbError::invalid_operation("dense_rank overflow"))?;
                }
                last_order = Some(ov);
            }
        }
        Ok(Value::from(dense_rank))
    }
}
