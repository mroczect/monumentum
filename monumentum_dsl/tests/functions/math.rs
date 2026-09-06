#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use monumentum_dsl::FunctionRegistry;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn call_scalar1(reg: &FunctionRegistry, name: &str, arg: f64) -> Result<Value, DbError> {
    let f = reg
        .get_scalar(name)
        .ok_or_else(|| DbError::unsupported(name))?;
    f.call(&[Value::try_from(arg)?])
}

fn call_scalar2(reg: &FunctionRegistry, name: &str, a: f64, b: f64) -> Result<Value, DbError> {
    let f = reg
        .get_scalar(name)
        .ok_or_else(|| DbError::unsupported(name))?;
    f.call(&[Value::try_from(a)?, Value::try_from(b)?])
}

fn assert_float_eq(result: Value, expected: f64, tol: f64) {
    let v = result.as_f64().expect("expected float");
    assert!(
        (v - expected).abs() < tol,
        "expected {}, got {}",
        expected,
        v
    );
}

#[test]
fn test_math_functions_registry_presence() {
    let reg = FunctionRegistry::new();
    for name in [
        "acos", "acosh", "asin", "asinh", "atan", "atan2", "atanh", "ceil", "ceiling", "cos",
        "cosh", "degrees", "exp", "floor", "ln", "log", "log10", "log2", "mod", "pi", "pow",
        "power", "radians", "sin", "sinh", "sqrt", "tan", "tanh", "trunc",
    ] {
        assert!(reg.get_scalar(name).is_some(), "missing scalar {}", name);
    }
}

#[test]
fn test_math_basic() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();

    assert_float_eq(call_scalar1(&reg, "sin", 0.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "cos", 0.0)?, 1.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "tan", 0.0)?, 0.0, 1e-9);

    assert_float_eq(call_scalar1(&reg, "asin", 0.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "acos", 1.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "atan", 0.0)?, 0.0, 1e-9);

    assert_float_eq(call_scalar1(&reg, "sinh", 0.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "cosh", 0.0)?, 1.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "tanh", 0.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "asinh", 0.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "acosh", 1.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "atanh", 0.0)?, 0.0, 1e-9);

    assert_float_eq(call_scalar1(&reg, "exp", 0.0)?, 1.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "ln", 1.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "log10", 1.0)?, 0.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "log2", 1.0)?, 0.0, 1e-9);

    assert_float_eq(call_scalar1(&reg, "ceil", 1.2)?, 2.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "floor", 1.8)?, 1.0, 1e-9);
    assert_float_eq(call_scalar1(&reg, "trunc", 1.8)?, 1.0, 1e-9);

    assert_float_eq(
        call_scalar1(&reg, "degrees", std::f64::consts::PI)?,
        180.0,
        1e-6,
    );
    assert_float_eq(
        call_scalar1(&reg, "radians", 180.0)?,
        std::f64::consts::PI,
        1e-6,
    );

    Ok(())
}

#[test]
fn test_atan2() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    assert_float_eq(
        call_scalar2(&reg, "atan2", 1.0, 1.0)?,
        std::f64::consts::FRAC_PI_4,
        1e-9,
    );
    assert_float_eq(
        call_scalar2(&reg, "atan2", -1.0, 1.0)?,
        -std::f64::consts::FRAC_PI_4,
        1e-9,
    );
    assert_float_eq(call_scalar2(&reg, "atan2", 0.0, 0.0)?, 0.0, 1e-9);
    Ok(())
}

#[test]
fn test_mod_fn() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    assert_float_eq(call_scalar2(&reg, "mod", 5.0, 2.0)?, 1.0, 1e-9);
    assert_float_eq(call_scalar2(&reg, "mod", -5.0, 2.0)?, -1.0, 1e-9);
    assert_float_eq(call_scalar2(&reg, "mod", 5.0, -2.0)?, 1.0, 1e-9);
    assert_eq!(call_scalar2(&reg, "mod", 5.0, 0.0)?, Value::Null);
    Ok(())
}

#[test]
fn test_pow_power() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    assert_float_eq(call_scalar2(&reg, "pow", 2.0, 3.0)?, 8.0, 1e-9);
    assert_float_eq(call_scalar2(&reg, "power", 2.0, 3.0)?, 8.0, 1e-9);
    assert_float_eq(call_scalar2(&reg, "pow", 4.0, 0.5)?, 2.0, 1e-9);
    Ok(())
}

#[test]
fn test_log_two_args() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    assert_float_eq(call_scalar2(&reg, "log", 2.0, 8.0)?, 3.0, 1e-9);
    assert_float_eq(call_scalar2(&reg, "log", 10.0, 100.0)?, 2.0, 1e-9);
    assert_eq!(call_scalar2(&reg, "log", 1.0, 10.0)?, Value::Null);
    assert_eq!(call_scalar2(&reg, "log", 0.0, 10.0)?, Value::Null);
    assert_eq!(call_scalar2(&reg, "log", -1.0, 10.0)?, Value::Null);
    Ok(())
}

#[test]
fn test_pi() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let f = reg
        .get_scalar("pi")
        .ok_or_else(|| DbError::unsupported("pi"))?;
    let result = f.call(&[])?;
    assert_float_eq(result, std::f64::consts::PI, 1e-9);
    Ok(())
}

#[test]
fn test_math_non_numeric_returns_null() -> Result<(), DbError> {
    let reg = FunctionRegistry::new();
    let f = reg
        .get_scalar("sin")
        .ok_or_else(|| DbError::unsupported("sin"))?;
    let result = f.call(&[Value::try_from("hello".to_string())?])?;
    assert_eq!(result, Value::Null);

    let f = reg
        .get_scalar("atan2")
        .ok_or_else(|| DbError::unsupported("atan2"))?;
    let result = f.call(&[Value::try_from("a".to_string())?, Value::try_from(1.0_f64)?])?;
    assert_eq!(result, Value::Null);
    Ok(())
}
