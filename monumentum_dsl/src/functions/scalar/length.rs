use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct LengthFunction;

impl ScalarFunction for LengthFunction {
    fn name(&self) -> &'static str {
        "length"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("length expects one argument"))?;
        let len = match arg {
            Value::Text(t) => i64::try_from(t.len())
                .map_err(|e| DbError::invalid_operation(format!("length overflow: {e}")))?,
            Value::Blob(b) => i64::try_from(b.len())
                .map_err(|e| DbError::invalid_operation(format!("length overflow: {e}")))?,
            Value::Null | Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | _ => {
                return Err(DbError::type_mismatch("length expects text or blob"));
            }
        };
        Ok(Value::from(len))
    }
}
