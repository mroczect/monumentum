use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct NthValueFunction;

impl WindowFunction for NthValueFunction {
    fn name(&self) -> &'static str {
        "nth_value"
    }

    fn evaluate(
        &self,
        partition: &[Row],
        _current_idx: usize,
        args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        if args.len() < 2 {
            return Err(DbError::invalid_operation(
                "nth_value requires two arguments: expr and N",
            ));
        }
        let n = args.get(1).and_then(Value::as_i64).ok_or_else(|| {
            DbError::type_mismatch("nth_value second argument must be an integer")
        })?;
        if n <= 0 {
            return Err(DbError::invalid_operation("nth_value N must be positive"));
        }
        let n_idx = usize::try_from(n.saturating_sub(1))
            .map_err(|_| DbError::invalid_operation("N too large"))?;
        if n_idx >= partition.len() {
            return Ok(Value::Null);
        }
        args.get(0)
            .cloned()
            .ok_or_else(|| DbError::invalid_operation("first argument missing"))
    }
}
