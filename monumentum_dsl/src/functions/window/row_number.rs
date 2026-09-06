use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct RowNumberFunction;

impl WindowFunction for RowNumberFunction {
    fn name(&self) -> &'static str {
        "row_number"
    }

    fn evaluate(
        &self,
        _partition: &[Row],
        current_idx: usize,
        _args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        let num = i64::try_from(current_idx)
            .map_err(|_e| DbError::invalid_operation("row index too large"))?
            .checked_add(1)
            .ok_or_else(|| DbError::invalid_operation("row number overflow"))?;
        Ok(Value::from(num))
    }
}
