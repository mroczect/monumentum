#![allow(clippy::all)]

use monumentum_core as _;
use monumentum_dsl::{FunctionRegistry, ScalarFunction};
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
use tempfile as _;
mod functions;

#[test]
fn test_registry_has_builtins() {
    let registry = FunctionRegistry::new();
    assert!(registry.get_scalar("upper").is_some());
    assert!(registry.get_scalar("lower").is_some());
    assert!(registry.get_scalar("length").is_some());
    assert!(registry.get_scalar("concat").is_some());
    assert!(registry.get_aggregate("count").is_some());
    assert!(registry.get_aggregate("sum").is_some());
    assert!(registry.get_aggregate("avg").is_some());
    assert!(registry.get_aggregate("min").is_some());
    assert!(registry.get_aggregate("max").is_some());
}

#[test]
fn test_registry_custom_scalar() -> Result<(), DbError> {
    #[derive(Debug, Clone, Copy)]
    struct CustomFunction;
    impl ScalarFunction for CustomFunction {
        fn name(&self) -> &'static str {
            "custom"
        }
        fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
            Ok(Value::from(42_i64))
        }
    }

    let mut registry = FunctionRegistry::new();
    registry.register_scalar(Box::new(CustomFunction));
    let f = registry
        .get_scalar("custom")
        .ok_or_else(|| DbError::unsupported("missing custom function"))?;
    let result = f.call(&[])?;
    assert_eq!(result.as_i64(), Some(42));
    Ok(())
}
