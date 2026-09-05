#![allow(clippy::all)]
use monumentum_dsl::{
    AcosFunction, AcoshFunction, AsinFunction, AsinhFunction, Atan2Function, AtanFunction,
    AtanhFunction, CeilFunction, CeilingFunction, CosFunction, CoshFunction, DegreesFunction,
    ExpFunction, FloorFunction, LnFunction, Log2Function, Log10Function, LogFunction, ModFunction,
    PiFunction, PowFunction, PowerFunction, RadiansFunction, ScalarFunction, SinFunction,
    SinhFunction, SqrtFunction, TanFunction, TanhFunction, TruncFunction,
};
use monumentum_handler::MonumentumError;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

fn fv(x: f64) -> Value {
    Value::try_from(x).expect("valid float")
}

fn assert_float(result: Value, expected: f64) -> Result<(), DbError> {
    match result {
        Value::Float(f) => {
            assert!((f.as_f64() - expected).abs() < 1e-12);
            Ok(())
        }
        other => Err(DbError::type_mismatch(format!(
            "expected float, got {:?}",
            other
        ))),
    }
}

#[test]
fn test_ceil() -> Result<(), DbError> {
    let f = CeilFunction;
    assert_float(f.call(&[fv(2.3)])?, 3.0)?;
    assert_float(f.call(&[Value::from(2_i64)])?, 2.0)?;
    assert_float(f.call(&[fv(-2.3)])?, -2.0)?;
    assert_float(f.call(&[Value::try_from("2.7".to_string())?])?, 3.0)?;
    assert_eq!(f.call(&[Value::Null])?, Value::Null);
    let err = f.call(&[]).unwrap_err();
    assert_eq!(
        err.kind(),
        monumentum_handler::error::ErrorKind::InvalidOperation
    );
    Ok(())
}

#[test]
fn test_ceiling_alias() -> Result<(), DbError> {
    let f = CeilingFunction;
    assert_float(f.call(&[fv(2.1)])?, 3.0)?;
    Ok(())
}

#[test]
fn test_floor() -> Result<(), DbError> {
    let f = FloorFunction;
    assert_float(f.call(&[fv(2.7)])?, 2.0)?;
    assert_float(f.call(&[fv(-2.1)])?, -3.0)?;
    Ok(())
}

#[test]
fn test_trunc() -> Result<(), DbError> {
    let f = TruncFunction;
    assert_float(f.call(&[fv(2.9)])?, 2.0)?;
    assert_float(f.call(&[fv(-2.9)])?, -2.0)?;
    Ok(())
}

#[test]
fn test_sqrt() -> Result<(), DbError> {
    let f = SqrtFunction;
    assert_float(f.call(&[fv(9.0)])?, 3.0)?;
    assert_eq!(f.call(&[fv(-1.0)])?, Value::Null);
    assert_float(f.call(&[Value::from(16_i64)])?, 4.0)?;
    Ok(())
}

#[test]
fn test_exp() -> Result<(), DbError> {
    let f = ExpFunction;
    assert_float(f.call(&[fv(0.0)])?, 1.0)?;
    let result = f.call(&[fv(1.0)])?;
    if let Value::Float(v) = result {
        assert!((v.as_f64() - std::f64::consts::E).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    Ok(())
}

#[test]
fn test_ln() -> Result<(), DbError> {
    let f = LnFunction;
    assert_float(f.call(&[fv(1.0)])?, 0.0)?;
    assert_eq!(f.call(&[fv(0.0)])?, Value::Null);
    assert_eq!(f.call(&[fv(-1.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_log10() -> Result<(), DbError> {
    let f = Log10Function;
    assert_float(f.call(&[fv(100.0)])?, 2.0)?;
    assert_eq!(f.call(&[fv(0.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_log2() -> Result<(), DbError> {
    let f = Log2Function;
    assert_float(f.call(&[fv(8.0)])?, 3.0)?;
    assert_eq!(f.call(&[fv(-1.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_trig_sin_cos_tan() -> Result<(), DbError> {
    let sin = SinFunction;
    let cos = CosFunction;
    let tan = TanFunction;
    assert_float(sin.call(&[fv(0.0)])?, 0.0)?;
    assert_float(cos.call(&[fv(0.0)])?, 1.0)?;
    assert_float(tan.call(&[fv(0.0)])?, 0.0)?;
    Ok(())
}

#[test]
fn test_asin_acos_atan() -> Result<(), DbError> {
    let asin = AsinFunction;
    let acos = AcosFunction;
    let atan = AtanFunction;
    assert_float(asin.call(&[fv(0.0)])?, 0.0)?;
    assert_float(acos.call(&[fv(1.0)])?, 0.0)?;
    assert_float(atan.call(&[fv(0.0)])?, 0.0)?;
    assert_eq!(asin.call(&[fv(2.0)])?, Value::Null);
    assert_eq!(acos.call(&[fv(-2.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_hyperbolic() -> Result<(), DbError> {
    let sinh = SinhFunction;
    let cosh = CoshFunction;
    let tanh = TanhFunction;
    assert_float(sinh.call(&[fv(0.0)])?, 0.0)?;
    assert_float(cosh.call(&[fv(0.0)])?, 1.0)?;
    assert_float(tanh.call(&[fv(0.0)])?, 0.0)?;
    Ok(())
}

#[test]
fn test_asinh_acosh_atanh() -> Result<(), DbError> {
    let asinh = AsinhFunction;
    let acosh = AcoshFunction;
    let atanh = AtanhFunction;
    assert_float(asinh.call(&[fv(0.0)])?, 0.0)?;
    assert_float(acosh.call(&[fv(1.0)])?, 0.0)?;
    assert_float(atanh.call(&[fv(0.0)])?, 0.0)?;
    assert_eq!(acosh.call(&[fv(0.5)])?, Value::Null);
    assert_eq!(atanh.call(&[fv(1.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_degrees_radians() -> Result<(), DbError> {
    let deg = DegreesFunction;
    let rad = RadiansFunction;
    assert_float(deg.call(&[fv(std::f64::consts::PI)])?, 180.0)?;
    assert_float(rad.call(&[fv(180.0)])?, std::f64::consts::PI)?;
    Ok(())
}

#[test]
fn test_pi() -> Result<(), DbError> {
    let pi = PiFunction;
    let result = pi.call(&[])?;
    assert_float(result, std::f64::consts::PI)?;
    Ok(())
}

#[test]
fn test_atan2() -> Result<(), DbError> {
    let f = Atan2Function;
    let result = f.call(&[fv(1.0), fv(1.0)])?;
    if let Value::Float(v) = result {
        assert!((v.as_f64() - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    } else {
        return Err(DbError::type_mismatch("expected float"));
    }
    assert_eq!(f.call(&[fv(1.0)])?, Value::Null);
    assert_eq!(f.call(&[])?, Value::Null);
    Ok(())
}

#[test]
fn test_mod() -> Result<(), DbError> {
    let f = ModFunction;
    assert_float(f.call(&[fv(5.0), fv(2.0)])?, 1.0)?;
    assert_float(f.call(&[fv(-5.0), fv(2.0)])?, -1.0)?;
    assert_eq!(f.call(&[fv(5.0), fv(0.0)])?, Value::Null);
    assert_eq!(f.call(&[fv(5.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_pow() -> Result<(), DbError> {
    let f = PowFunction;
    assert_float(f.call(&[fv(2.0), fv(3.0)])?, 8.0)?;
    assert_float(f.call(&[fv(4.0), fv(0.5)])?, 2.0)?;
    assert_eq!(f.call(&[fv(2.0)])?, Value::Null);
    Ok(())
}

#[test]
fn test_power_alias() -> Result<(), DbError> {
    let f = PowerFunction;
    assert_float(f.call(&[fv(2.0), fv(10.0)])?, 1024.0)?;
    Ok(())
}

#[test]
fn test_log_one_arg_log10() -> Result<(), DbError> {
    let f = LogFunction;
    assert_float(f.call(&[fv(1000.0)])?, 3.0)?;
    assert_eq!(f.call(&[fv(0.0)])?, Value::Null);
    assert_float(f.call(&[fv(2.0), fv(8.0)])?, 3.0)?;
    assert_eq!(f.call(&[fv(0.0), fv(8.0)])?, Value::Null);
    assert_eq!(f.call(&[fv(1.0), fv(8.0)])?, Value::Null);
    assert_eq!(f.call(&[fv(2.0), fv(0.0)])?, Value::Null);
    assert_eq!(f.call(&[])?, Value::Null);
    assert_eq!(f.call(&[fv(1.0), fv(2.0), fv(3.0)])?, Value::Null);
    Ok(())
}
