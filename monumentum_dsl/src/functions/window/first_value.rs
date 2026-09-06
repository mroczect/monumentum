use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct FirstValueFunction;

impl WindowFunction for FirstValueFunction {
    fn name(&self) -> &'static str {
        "first_value"
    }

    fn evaluate(
        &self,
        _partition: &[Row],
        _current_idx: usize,
        args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        args.first()
            .cloned()
            .ok_or_else(|| DbError::invalid_operation("first_value requires at least one argument"))
    }
}
