extern crate alloc;

use monumentum_core as _;

#[cfg(test)]
use tempfile as _;

mod functions;
mod macros;
mod query;

pub use functions::{
    Accumulator, AggregateFunction, AvgFunction, ConcatFunction, CountFunction, FunctionRegistry,
    GroupConcatFunction, LengthFunction, LowerFunction, MaxFunction, MedianFunction, MinFunction,
    PercentileContFunction, PercentileDiscFunction, ScalarFunction, StringAggFunction, SumFunction,
    TotalFunction, UpperFunction,
};
pub use query::{ProjectedQueryBuilder, QueryBuilder};

pub mod prelude {
    pub use crate::functions::{Accumulator, AggregateFunction, FunctionRegistry, ScalarFunction};
    pub use crate::{ProjectedQueryBuilder, QueryBuilder};
    pub use crate::{query, query_project};
}
