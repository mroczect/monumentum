use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[derive(Debug, Clone, Copy)]
pub struct ConcatFunction;

impl ScalarFunction for ConcatFunction {
    fn name(&self) -> &'static str {
        "concat"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let mut result = String::new();
        for arg in args {
            if let Some(s) = arg.as_str() {
                result.push_str(s);
            } else {
                return Err(DbError::type_mismatch("concat expects text arguments"));
            }
        }
        Value::try_from(result)
    }
}
