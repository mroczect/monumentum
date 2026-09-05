use alloc::boxed::Box;

use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

mod aggregate;
mod registry;
mod scalar;

pub use registry::FunctionRegistry;
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
