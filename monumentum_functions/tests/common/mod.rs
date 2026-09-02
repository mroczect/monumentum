mod and;
mod average;
mod concat;
mod if_fn;
mod len;
mod lower;
mod max;
mod min;
mod not;
mod or;
mod sum;
mod trim;
mod upper;

use monumentum_db::core::value::Value;
use monumentum_db::types::Float;
use monumentum_functions::register_all;
use monumentum_query::formula::{FormulaError, FunctionRegistry};

#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
fn call_function(name: &str, args: &[Value]) -> Result<Value, FormulaError> {
    let mut registry = FunctionRegistry::new();
    register_all(&mut registry);
    registry.call(name, args)
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
fn unwrap_result<T>(result: Result<T, FormulaError>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("unexpected error in test: {e}"),
    }
}

#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
fn float_value(value: f64) -> Value {
    Value::Float(Float::try_new(value).unwrap())
}
