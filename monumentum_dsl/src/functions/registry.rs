use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;

use crate::functions::{
    AggregateFunction, ScalarFunction,
    aggregate::{
        avg::AvgFunction,
        count::CountFunction,
        extras::{
            GroupConcatFunction, MedianFunction, PercentileContFunction, PercentileDiscFunction,
            StringAggFunction, TotalFunction,
        },
        max::MaxFunction,
        min::MinFunction,
        sum::SumFunction,
    },
    scalar::{
        concat::ConcatFunction,
        length::LengthFunction,
        lower::LowerFunction,
        math::{
            AcosFunction, AcoshFunction, AsinFunction, AsinhFunction, Atan2Function, AtanFunction,
            AtanhFunction, CeilFunction, CeilingFunction, CosFunction, CoshFunction,
            DegreesFunction, ExpFunction, FloorFunction, LnFunction, Log2Function, Log10Function,
            LogFunction, ModFunction, PiFunction, PowFunction, PowerFunction, RadiansFunction,
            SinFunction, SinhFunction, SqrtFunction, TanFunction, TanhFunction, TruncFunction,
        },
        upper::UpperFunction,
    },
};

#[derive(Default)]
pub struct FunctionRegistry {
    scalars: BTreeMap<String, Box<dyn ScalarFunction>>,
    aggregates: BTreeMap<String, Box<dyn AggregateFunction>>,
}

impl FunctionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::default();

        registry.register_scalar(Box::new(UpperFunction));
        registry.register_scalar(Box::new(LowerFunction));
        registry.register_scalar(Box::new(LengthFunction));
        registry.register_scalar(Box::new(ConcatFunction));
        registry.register_scalar(Box::new(AcosFunction));
        registry.register_scalar(Box::new(AcoshFunction));
        registry.register_scalar(Box::new(AsinFunction));
        registry.register_scalar(Box::new(AsinhFunction));
        registry.register_scalar(Box::new(AtanFunction));
        registry.register_scalar(Box::new(Atan2Function));
        registry.register_scalar(Box::new(AtanhFunction));
        registry.register_scalar(Box::new(CeilFunction));
        registry.register_scalar(Box::new(CeilingFunction));
        registry.register_scalar(Box::new(CosFunction));
        registry.register_scalar(Box::new(CoshFunction));
        registry.register_scalar(Box::new(DegreesFunction));
        registry.register_scalar(Box::new(ExpFunction));
        registry.register_scalar(Box::new(FloorFunction));
        registry.register_scalar(Box::new(LnFunction));
        registry.register_scalar(Box::new(LogFunction));
        registry.register_scalar(Box::new(Log10Function));
        registry.register_scalar(Box::new(Log2Function));
        registry.register_scalar(Box::new(ModFunction));
        registry.register_scalar(Box::new(PiFunction));
        registry.register_scalar(Box::new(PowFunction));
        registry.register_scalar(Box::new(PowerFunction));
        registry.register_scalar(Box::new(RadiansFunction));
        registry.register_scalar(Box::new(SinFunction));
        registry.register_scalar(Box::new(SinhFunction));
        registry.register_scalar(Box::new(SqrtFunction));
        registry.register_scalar(Box::new(TanFunction));
        registry.register_scalar(Box::new(TanhFunction));
        registry.register_scalar(Box::new(TruncFunction));

        registry.register_aggregate(Box::new(CountFunction));
        registry.register_aggregate(Box::new(SumFunction));
        registry.register_aggregate(Box::new(AvgFunction));
        registry.register_aggregate(Box::new(MinFunction));
        registry.register_aggregate(Box::new(MaxFunction));
        registry.register_aggregate(Box::new(GroupConcatFunction::default()));
        registry.register_aggregate(Box::new(StringAggFunction::default()));
        registry.register_aggregate(Box::new(TotalFunction));
        registry.register_aggregate(Box::new(MedianFunction));
        registry.register_aggregate(Box::new(PercentileContFunction::new(0.5)));
        registry.register_aggregate(Box::new(PercentileDiscFunction::new(0.5)));

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
