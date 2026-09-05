extern crate alloc;

use monumentum_core as _;

#[cfg(test)]
use tempfile as _;

mod functions;
mod macros;
mod query;

pub use functions::{
    Accumulator, AggregateFunction, AvgFunction, ConcatFunction, CountFunction, FunctionRegistry,
    LengthFunction, LowerFunction, MaxFunction, MinFunction, ScalarFunction, SumFunction,
    UpperFunction,
};
pub use query::{ProjectedQueryBuilder, QueryBuilder};

pub mod prelude {
    pub use crate::functions::{Accumulator, AggregateFunction, FunctionRegistry, ScalarFunction};
    pub use crate::{ProjectedQueryBuilder, QueryBuilder};
    pub use crate::{query, query_project};
}
