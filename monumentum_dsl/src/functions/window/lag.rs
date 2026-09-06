use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct LagFunction;

impl WindowFunction for LagFunction {
    fn name(&self) -> &'static str {
        "lag"
    }

    fn evaluate(
        &self,
        _partition: &[Row],
        current_idx: usize,
        args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "lag requires at least one argument",
            ));
        }

        let offset = if args.len() >= 2 {
            args.get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| DbError::type_mismatch("lag offset must be an integer"))?
        } else {
            1_i64
        };

        if offset < 0 {
            return Err(DbError::invalid_operation(
                "lag offset must be non-negative",
            ));
        }

        let current_i64 = i64::try_from(current_idx)
            .map_err(|_e| DbError::invalid_operation("current index too large"))?;
        let target_idx = current_i64
            .checked_sub(offset)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;

        if target_idx < 0 {
            return if args.len() >= 3 {
                args.get(2)
                    .cloned()
                    .ok_or_else(|| DbError::invalid_operation("missing default value"))
            } else {
                Ok(Value::Null)
            };
        }

        args.first()
            .cloned()
            .ok_or_else(|| DbError::invalid_operation("lag first argument missing"))
    }
}
