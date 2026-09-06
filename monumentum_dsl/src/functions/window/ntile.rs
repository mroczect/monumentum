use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::window::WindowFunction;

#[derive(Debug, Clone, Copy)]
pub struct NtileFunction;

impl WindowFunction for NtileFunction {
    fn name(&self) -> &'static str {
        "ntile"
    }

    fn evaluate(
        &self,
        partition: &[Row],
        current_idx: usize,
        args: &[Value],
        _order_values: &[Option<Value>],
    ) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("ntile requires one argument"))?;
        let bucket_count_i64 = arg
            .as_i64()
            .ok_or_else(|| DbError::type_mismatch("ntile argument must be an integer"))?;
        if bucket_count_i64 <= 0 {
            return Err(DbError::invalid_operation(
                "ntile argument must be positive",
            ));
        }
        let bucket_count = usize::try_from(bucket_count_i64)
            .map_err(|_e| DbError::invalid_operation("bucket count too large"))?;
        let partition_len = partition.len();
        if partition_len == 0 {
            return Ok(Value::Null);
        }

        let base = partition_len
            .checked_div(bucket_count)
            .ok_or_else(|| DbError::invalid_operation("division by zero"))?;
        let remainder = partition_len
            .checked_rem(bucket_count)
            .ok_or_else(|| DbError::invalid_operation("division by zero"))?;

        let ok_or = |opt: Option<usize>| {
            opt.ok_or_else(|| DbError::invalid_operation("ntile calculation overflow"))
        };

        let larger_group_size = ok_or(base.checked_add(1))?;
        let threshold = larger_group_size
            .checked_mul(remainder)
            .ok_or_else(|| DbError::invalid_operation("multiplication overflow"))?;

        let bucket_idx = if current_idx < threshold {
            ok_or(
                current_idx
                    .checked_div(larger_group_size)
                    .and_then(|v| v.checked_add(1)),
            )?
        } else {
            let adjusted_idx = ok_or(current_idx.checked_sub(threshold))?;
            ok_or(
                adjusted_idx
                    .checked_div(base)
                    .and_then(|v| v.checked_add(remainder))
                    .and_then(|v| v.checked_add(1)),
            )?
        };

        let bucket_value = i64::try_from(bucket_idx)
            .map_err(|_e| DbError::invalid_operation("bucket index too large"))?;
        Ok(Value::from(bucket_value))
    }
}
