use crate::functions::extras::percentile_cont::PercentileContAccumulator;
use crate::functions::{Accumulator, AggregateFunction};
use alloc::boxed::Box;

#[derive(Debug, Clone, Copy)]
pub struct MedianFunction;

impl AggregateFunction for MedianFunction {
    fn name(&self) -> &'static str {
        "median"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(PercentileContAccumulator::new(0.5_f64))
    }
}
