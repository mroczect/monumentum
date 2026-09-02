use monumentum_db::core::value::Value;
use monumentum_query::formula::{FormulaError, FunctionRegistry};

fn dummy_fn(_args: &[Value]) -> Result<Value, FormulaError> {
    Ok(Value::Integer(42.into()))
}

#[test]
fn registry_new_is_empty() {
    let registry = FunctionRegistry::new();
    assert!(!registry.contains("FOO"));
}

#[test]
fn registry_register_and_contains_case_insensitive() {
    let mut registry = FunctionRegistry::new();
    registry.register("foo", dummy_fn);
    assert!(registry.contains("FOO"));
    assert!(registry.contains("foo"));
}

#[test]
fn registry_call_function() {
    let mut registry = FunctionRegistry::new();
    registry.register("BAR", dummy_fn);
    let result = registry.call("bar", &[]).unwrap();
    assert_eq!(result, Value::Integer(42.into()));
}

#[test]
fn registry_call_unknown() {
    let registry = FunctionRegistry::new();
    let err = registry.call("BAZ", &[]).unwrap_err();
    assert_eq!(err, FormulaError::UnknownFunction("BAZ".to_string()));
}

#[test]
fn registry_remove() {
    let mut registry = FunctionRegistry::new();
    registry.register("TEMP", dummy_fn);
    assert!(registry.contains("TEMP"));
    let removed = registry.remove("temp");
    assert!(removed.is_some());
    assert!(!registry.contains("TEMP"));
    assert!(registry.remove("nonexistent").is_none());
}

#[test]
fn registry_register_overwrites() {
    let mut registry = FunctionRegistry::new();
    registry.register("FUNC", dummy_fn);
    let result1 = registry.call("FUNC", &[]).unwrap();
    assert_eq!(result1, Value::Integer(42.into()));

    fn other_fn(_: &[Value]) -> Result<Value, FormulaError> {
        Ok(Value::Boolean(true))
    }
    registry.register("FUNC", other_fn);
    let result2 = registry.call("FUNC", &[]).unwrap();
    assert_eq!(result2, Value::Boolean(true));
}
