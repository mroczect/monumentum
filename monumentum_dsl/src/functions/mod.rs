use alloc::boxed::Box;

use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

#[allow(clippy::redundant_pub_crate)]
pub(crate) mod aggregate;
mod registry;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod scalar;

pub use aggregate::{
    avg::AvgFunction,
    count::CountFunction,
    extras::{
        GroupConcatFunction, MedianFunction, PercentileContFunction, PercentileDiscFunction,
        StringAggFunction, TotalFunction,
    },
    max::MaxFunction,
    min::MinFunction,
    sum::SumFunction,
};
pub use registry::FunctionRegistry;
pub use scalar::{
    concat::ConcatFunction,
    length::LengthFunction,
    lower::LowerFunction,
    math::{
        AcosFunction, AcoshFunction, AsinFunction, AsinhFunction, Atan2Function, AtanFunction,
        AtanhFunction, CeilFunction, CeilingFunction, CosFunction, CoshFunction, DegreesFunction,
        ExpFunction, FloorFunction, LnFunction, Log2Function, Log10Function, LogFunction,
        ModFunction, PiFunction, PowFunction, PowerFunction, RadiansFunction, SinFunction,
        SinhFunction, SqrtFunction, TanFunction, TanhFunction, TruncFunction,
    },
    upper::UpperFunction,
};

pub trait ScalarFunction: Send + Sync {
    fn name(&self) -> &'static str;
    fn call(&self, args: &[Value]) -> Result<Value, DbError>;
}

pub trait AggregateFunction: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&self) -> Box<dyn Accumulator>;
}

pub trait Accumulator: Send + Sync {
    fn update(&mut self, value: &Value) -> Result<(), DbError>;
    fn finish(self: Box<Self>) -> Result<Value, DbError>;
}
