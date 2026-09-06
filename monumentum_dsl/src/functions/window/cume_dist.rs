use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct CumeDistFunction;

impl WindowFunction for CumeDistFunction {
    fn name(&self) -> &'static str {
        "cume_dist"
    }

    fn evaluate(
        &self,
        partition: &[Row],
        current_idx: usize,
        _args: &[Value],
        order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        let n = partition.len();
        if n == 0 {
            return Ok(Value::Null);
        }

        let count = if order_values.is_empty() {
            n
        } else {
            let current_order = order_values
                .get(current_idx)
                .ok_or_else(|| DbError::invalid_operation("current index out of bounds"))?;
            order_values
                .iter()
                .filter(|ov| *ov <= current_order)
                .count()
        };

        #[allow(clippy::cast_precision_loss)]
        let cume = count as f64 / n as f64;
        Value::try_from(cume)
    }
}
