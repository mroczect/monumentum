use monumentum_handler::core::value::Value;
#[allow(clippy::cast_precision_loss)]
pub(crate) fn value_to_f64(value: &Value) -> Option<f64> {
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

        impl crate::functions::ScalarFunction for $struct_name {
            fn name(&self) -> &'static str {
                $name
            }

            fn call(
                &self,
                args: &[monumentum_handler::core::value::Value],
            ) -> Result<monumentum_handler::core::value::Value, monumentum_handler::error::DbError>
            {
                let arg = args.first().ok_or_else(|| {
                    monumentum_handler::error::DbError::invalid_operation(
                        "math function expects one argument",
                    )
                })?;
                let Some(x) = crate::functions::math::value_to_f64(arg) else {
                    return Ok(monumentum_handler::core::value::Value::Null);
                };
                let result = $body(x);
                if result.is_finite() {
                    monumentum_handler::core::value::Value::try_from(result)
                } else {
                    Ok(monumentum_handler::core::value::Value::Null)
                }
            }
        }
    };
}

pub mod acos;
pub mod acosh;
pub mod asin;
pub mod asinh;
pub mod atan;
pub mod atan2;
pub mod atanh;
pub mod ceil;
pub mod ceiling;
pub mod cos;
pub mod cosh;
pub mod degrees;
pub mod exp;
pub mod floor;
pub mod ln;
pub mod log;
pub mod log10;
pub mod log2;
pub mod mod_fn;
pub mod pi;
pub mod pow;
pub mod power;
pub mod radians;
pub mod sin;
pub mod sinh;
pub mod sqrt;
pub mod tan;
pub mod tanh;
pub mod trunc;

pub use self::acos::AcosFunction;
pub use self::acosh::AcoshFunction;
pub use self::asin::AsinFunction;
pub use self::asinh::AsinhFunction;
pub use self::atan::AtanFunction;
pub use self::atan2::Atan2Function;
pub use self::atanh::AtanhFunction;
pub use self::ceil::CeilFunction;
pub use self::ceiling::CeilingFunction;
pub use self::cos::CosFunction;
pub use self::cosh::CoshFunction;
pub use self::degrees::DegreesFunction;
pub use self::exp::ExpFunction;
pub use self::floor::FloorFunction;
pub use self::ln::LnFunction;
pub use self::log::LogFunction;
pub use self::log2::Log2Function;
pub use self::log10::Log10Function;
pub use self::mod_fn::ModFunction;
pub use self::pi::PiFunction;
pub use self::pow::PowFunction;
pub use self::power::PowerFunction;
pub use self::radians::RadiansFunction;
pub use self::sin::SinFunction;
pub use self::sinh::SinhFunction;
pub use self::sqrt::SqrtFunction;
pub use self::tan::TanFunction;
pub use self::tanh::TanhFunction;
pub use self::trunc::TruncFunction;
