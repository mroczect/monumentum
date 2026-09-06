#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::{extract_path, parse_json};

#[derive(Debug, Clone, Copy)]
pub struct JsonArrowDoubleFunction;

impl ScalarFunction for JsonArrowDoubleFunction {
    fn name(&self) -> &'static str {
        "->>"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Err(DbError::invalid_operation(
                "->> operator requires two arguments",
            ));
        }
        let json_text = args[0]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("left operand must be text"))?;
        let root = parse_json(json_text)?;
        let path = args[1]
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("right operand must be text"))?;
        let result = extract_path(&root, path)?;
        match result {
            Some(jv) => jv.to_sql_value(),
            None => Ok(Value::Null),
        }
    }
}
