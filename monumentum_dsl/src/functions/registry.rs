use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;

use super::{AggregateFunction, ScalarFunction};

#[derive(Default)]
pub struct FunctionRegistry {
    scalars: BTreeMap<String, Box<dyn ScalarFunction>>,
    aggregates: BTreeMap<String, Box<dyn AggregateFunction>>,
}

impl FunctionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_scalar(Box::new(super::UpperFunction));
        registry.register_scalar(Box::new(super::LowerFunction));
        registry.register_scalar(Box::new(super::LengthFunction));
        registry.register_scalar(Box::new(super::ConcatFunction));
        registry.register_aggregate(Box::new(super::CountFunction));
        registry.register_aggregate(Box::new(super::SumFunction));
        registry.register_aggregate(Box::new(super::AvgFunction));
        registry.register_aggregate(Box::new(super::MinFunction));
        registry.register_aggregate(Box::new(super::MaxFunction));
        registry
    }

    pub fn register_scalar(&mut self, f: Box<dyn ScalarFunction>) {
        let _ = self.scalars.insert(f.name().to_string(), f);
    }

    pub fn register_aggregate(&mut self, f: Box<dyn AggregateFunction>) {
        let _ = self.aggregates.insert(f.name().to_string(), f);
    }

    #[must_use]
    pub fn get_scalar(&self, name: &str) -> Option<&dyn ScalarFunction> {
        self.scalars.get(name).map(Box::as_ref)
    }

    #[must_use]
    pub fn get_aggregate(&self, name: &str) -> Option<&dyn AggregateFunction> {
        self.aggregates.get(name).map(Box::as_ref)
    }
}

impl fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("scalars", &self.scalars.keys().collect::<Vec<_>>())
            .field("aggregates", &self.aggregates.keys().collect::<Vec<_>>())
            .finish()
    }
}
