use core::f64::consts::PI;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;

#[allow(clippy::cast_precision_loss)]
fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(i) => Some(i.as_i64() as f64),
        Value::Float(f) => Some(f.as_f64()),
        Value::Text(t) => t.as_str().parse::<f64>().ok(),
        Value::Null | Value::Blob(_) | Value::Boolean(_) | _ => None,
    }
}

macro_rules! define_math_fn {
    ($struct_name:ident, $name:expr, $body:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $struct_name;

        impl ScalarFunction for $struct_name {
            fn name(&self) -> &'static str {
                $name
            }

            fn call(&self, args: &[Value]) -> Result<Value, DbError> {
                let arg = args.first().ok_or_else(|| {
                    DbError::invalid_operation("math function expects one argument")
                })?;
                let Some(x) = value_to_f64(arg) else {
                    return Ok(Value::Null);
                };
                let result = $body(x);
                if result.is_finite() {
                    Value::try_from(result)
                } else {
                    Ok(Value::Null)
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct PiFunction;

impl ScalarFunction for PiFunction {
    fn name(&self) -> &'static str {
        "pi"
    }

    fn call(&self, _args: &[Value]) -> Result<Value, DbError> {
        Value::try_from(PI)
    }
}

define_math_fn!(CeilFunction, "ceil", |x: f64| x.ceil());
define_math_fn!(CeilingFunction, "ceiling", |x: f64| x.ceil());
define_math_fn!(FloorFunction, "floor", |x: f64| x.floor());
define_math_fn!(TruncFunction, "trunc", |x: f64| x.trunc());
define_math_fn!(SqrtFunction, "sqrt", |x: f64| x.sqrt());
define_math_fn!(ExpFunction, "exp", |x: f64| x.exp());
define_math_fn!(LnFunction, "ln", |x: f64| x.ln());
define_math_fn!(Log10Function, "log10", |x: f64| x.log10());
define_math_fn!(Log2Function, "log2", |x: f64| x.log2());
define_math_fn!(SinFunction, "sin", |x: f64| x.sin());
define_math_fn!(CosFunction, "cos", |x: f64| x.cos());
define_math_fn!(TanFunction, "tan", |x: f64| x.tan());
define_math_fn!(AsinFunction, "asin", |x: f64| x.asin());
define_math_fn!(AcosFunction, "acos", |x: f64| x.acos());
define_math_fn!(AtanFunction, "atan", |x: f64| x.atan());
define_math_fn!(SinhFunction, "sinh", |x: f64| x.sinh());
define_math_fn!(CoshFunction, "cosh", |x: f64| x.cosh());
define_math_fn!(TanhFunction, "tanh", |x: f64| x.tanh());
define_math_fn!(AsinhFunction, "asinh", |x: f64| x.asinh());
define_math_fn!(AcoshFunction, "acosh", |x: f64| x.acosh());
define_math_fn!(AtanhFunction, "atanh", |x: f64| x.atanh());
define_math_fn!(DegreesFunction, "degrees", |x: f64| x.to_degrees());
define_math_fn!(RadiansFunction, "radians", |x: f64| x.to_radians());

#[derive(Debug, Clone, Copy)]
pub struct Atan2Function;

impl ScalarFunction for Atan2Function {
    fn name(&self) -> &'static str {
        "atan2"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let y = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
        let x = args.get(1).and_then(value_to_f64).unwrap_or(0.0_f64);
        let result = y.atan2(x);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModFunction;

impl ScalarFunction for ModFunction {
    fn name(&self) -> &'static str {
        "mod"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let x = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
        let y = args.get(1).and_then(value_to_f64).unwrap_or(0.0_f64);
        if y == 0.0 {
            return Ok(Value::Null);
        }
        let result = (x / y).trunc().mul_add(-y, x);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowFunction;

impl ScalarFunction for PowFunction {
    fn name(&self) -> &'static str {
        "pow"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let x = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
        let y = args.get(1).and_then(value_to_f64).unwrap_or(0.0_f64);
        let result = x.powf(y);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerFunction;

impl ScalarFunction for PowerFunction {
    fn name(&self) -> &'static str {
        "power"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        if args.len() != 2 {
            return Ok(Value::Null);
        }
        let x = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
        let y = args.get(1).and_then(value_to_f64).unwrap_or(0.0_f64);
        let result = x.powf(y);
        if result.is_finite() {
            Value::try_from(result)
        } else {
            Ok(Value::Null)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LogFunction;

impl ScalarFunction for LogFunction {
    fn name(&self) -> &'static str {
        "log"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        match args.len() {
            1 => {
                let x = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
                let result = x.log10();
                if result.is_finite() {
                    Value::try_from(result)
                } else {
                    Ok(Value::Null)
                }
            }
            2 => {
                let b = args.first().and_then(value_to_f64).unwrap_or(0.0_f64);
                let x = args.get(1).and_then(value_to_f64).unwrap_or(0.0_f64);
                if b <= 0.0 || (b - 1.0).abs() < f64::EPSILON || x <= 0.0 {
                    return Ok(Value::Null);
                }
                let result = x.log(b);
                if result.is_finite() {
                    Value::try_from(result)
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Ok(Value::Null),
        }
    }
}
