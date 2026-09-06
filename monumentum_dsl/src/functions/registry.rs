use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use std::sync::OnceLock;

use crate::functions::{
    AggregateFunction, ScalarFunction,
    aggregate::{
        avg::AvgFunction,
        count::CountFunction,
        max::MaxFunction,
        min::MinFunction,
        sum::SumFunction,
    },
    extras::{
        GroupConcatFunction, MedianFunction, PercentileContFunction, PercentileDiscFunction,
        StringAggFunction, TotalFunction,
    },
    datetime::{
        DateFunction, DatetimeFunction, JuliandayFunction, StrftimeFunction, TimeFunction,
        TimediffFunction, UnixepochFunction,
    },
    math::{
        AcosFunction, AcoshFunction, AsinFunction, AsinhFunction, Atan2Function, AtanFunction,
        AtanhFunction, CeilFunction, CeilingFunction, CosFunction, CoshFunction, DegreesFunction,
        ExpFunction, FloorFunction, LnFunction, Log2Function, Log10Function, LogFunction,
        ModFunction, PiFunction, PowFunction, PowerFunction, RadiansFunction, SinFunction,
        SinhFunction, SqrtFunction, TanFunction, TanhFunction, TruncFunction,
    },
    scalar::{
        concat::ConcatFunction, length::LengthFunction, lower::LowerFunction, upper::UpperFunction,
    },
};

static DEFAULT_REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();

#[derive(Default)]
pub struct FunctionRegistry {
    scalars: BTreeMap<String, Box<dyn ScalarFunction>>,
    aggregates: BTreeMap<String, Box<dyn AggregateFunction>>,
}

impl FunctionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::default();

        let _ = registry.register_scalar(Box::new(UpperFunction));
        let _ = registry.register_scalar(Box::new(LowerFunction));
        let _ = registry.register_scalar(Box::new(LengthFunction));
        let _ = registry.register_scalar(Box::new(ConcatFunction));

        let _ = registry.register_scalar(Box::new(AcosFunction));
        let _ = registry.register_scalar(Box::new(AcoshFunction));
        let _ = registry.register_scalar(Box::new(AsinFunction));
        let _ = registry.register_scalar(Box::new(AsinhFunction));
        let _ = registry.register_scalar(Box::new(AtanFunction));
        let _ = registry.register_scalar(Box::new(Atan2Function));
        let _ = registry.register_scalar(Box::new(AtanhFunction));
        let _ = registry.register_scalar(Box::new(CeilFunction));
        let _ = registry.register_scalar(Box::new(CeilingFunction));
        let _ = registry.register_scalar(Box::new(CosFunction));
        let _ = registry.register_scalar(Box::new(CoshFunction));
        let _ = registry.register_scalar(Box::new(DegreesFunction));
        let _ = registry.register_scalar(Box::new(ExpFunction));
        let _ = registry.register_scalar(Box::new(FloorFunction));
        let _ = registry.register_scalar(Box::new(LnFunction));
        let _ = registry.register_scalar(Box::new(LogFunction));
        let _ = registry.register_scalar(Box::new(Log10Function));
        let _ = registry.register_scalar(Box::new(Log2Function));
        let _ = registry.register_scalar(Box::new(ModFunction));
        let _ = registry.register_scalar(Box::new(PiFunction));
        let _ = registry.register_scalar(Box::new(PowFunction));
        let _ = registry.register_scalar(Box::new(PowerFunction));
        let _ = registry.register_scalar(Box::new(RadiansFunction));
        let _ = registry.register_scalar(Box::new(SinFunction));
        let _ = registry.register_scalar(Box::new(SinhFunction));
        let _ = registry.register_scalar(Box::new(SqrtFunction));
        let _ = registry.register_scalar(Box::new(TanFunction));
        let _ = registry.register_scalar(Box::new(TanhFunction));
        let _ = registry.register_scalar(Box::new(TruncFunction));

        let _ = registry.register_scalar(Box::new(DateFunction));
        let _ = registry.register_scalar(Box::new(TimeFunction));
        let _ = registry.register_scalar(Box::new(DatetimeFunction));
        let _ = registry.register_scalar(Box::new(JuliandayFunction));
        let _ = registry.register_scalar(Box::new(UnixepochFunction));
        let _ = registry.register_scalar(Box::new(StrftimeFunction));
        let _ = registry.register_scalar(Box::new(TimediffFunction));

        let _ = registry.register_aggregate(Box::new(CountFunction));
        let _ = registry.register_aggregate(Box::new(SumFunction));
        let _ = registry.register_aggregate(Box::new(AvgFunction));
        let _ = registry.register_aggregate(Box::new(MinFunction));
        let _ = registry.register_aggregate(Box::new(MaxFunction));

        let _ = registry.register_aggregate(Box::new(GroupConcatFunction::default()));
        let _ = registry.register_aggregate(Box::new(StringAggFunction::default()));
        let _ = registry.register_aggregate(Box::new(TotalFunction));
        let _ = registry.register_aggregate(Box::new(MedianFunction));
        let _ = registry.register_aggregate(Box::new(PercentileContFunction::new(0.5)));
        let _ = registry.register_aggregate(Box::new(PercentileDiscFunction::new(0.5)));

        registry
    }

    #[must_use]
    pub fn global() -> &'static Self {
        DEFAULT_REGISTRY.get_or_init(Self::new)
    }

    pub fn register_scalar(
        &mut self,
        f: Box<dyn ScalarFunction>,
    ) -> Option<Box<dyn ScalarFunction>> {
        self.scalars.insert(f.name().to_string(), f)
    }

    pub fn register_aggregate(
        &mut self,
        f: Box<dyn AggregateFunction>,
    ) -> Option<Box<dyn AggregateFunction>> {
        self.aggregates.insert(f.name().to_string(), f)
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
