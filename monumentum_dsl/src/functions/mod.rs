use alloc::boxed::Box;

use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;
pub mod json;
pub use json::*;
pub mod datetime;
pub use datetime::*;
pub mod window;
pub use window::*;
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
pub mod math;
pub use math::*;

pub use scalar::{
    concat::ConcatFunction, length::LengthFunction, lower::LowerFunction, upper::UpperFunction,
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
