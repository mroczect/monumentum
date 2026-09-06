use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct LeadFunction;

impl WindowFunction for LeadFunction {
    fn name(&self) -> &'static str {
        "lead"
    }

    fn evaluate(
        &self,
        partition: &[Row],
        current_idx: usize,
        args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        if args.is_empty() {
            return Err(DbError::invalid_operation(
                "lead requires at least one argument",
            ));
        }

        let offset = if args.len() >= 2 {
            args.get(1)
                .and_then(Value::as_i64)
                .ok_or_else(|| DbError::type_mismatch("lead offset must be an integer"))?
        } else {
            1_i64
        };

        if offset < 0 {
            return Err(DbError::invalid_operation(
                "lead offset must be non-negative",
            ));
        }

        let current_i64 = i64::try_from(current_idx)
            .map_err(|_| DbError::invalid_operation("current index too large"))?;
        let target_idx = current_i64
            .checked_add(offset)
            .ok_or_else(|| DbError::invalid_operation("offset overflow"))?;
        let partition_len = i64::try_from(partition.len())
            .map_err(|_| DbError::invalid_operation("partition too large"))?;

        if target_idx >= partition_len {
            return if args.len() >= 3 {
                Ok(args.get(2).cloned().unwrap_or(Value::Null))
            } else {
                Ok(Value::Null)
            };
        }

        Ok(args[0].clone())
    }
}
