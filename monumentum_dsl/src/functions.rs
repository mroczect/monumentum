use alloc::collections::BTreeMap;
use core::fmt;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

pub trait ScalarFunction: Send + Sync {
    fn name(&self) -> &str;
    fn call(&self, args: &[Value]) -> Result<Value, DbError>;
}

pub trait AggregateFunction: Send + Sync {
    fn name(&self) -> &str;
    fn init(&self) -> Box<dyn Accumulator>;
}

pub trait Accumulator: Send + Sync {
    fn update(&mut self, value: &Value) -> Result<(), DbError>;
    fn finish(self: Box<Self>) -> Result<Value, DbError>;
}

#[derive(Default)]
pub struct FunctionRegistry {
    scalars: BTreeMap<String, Box<dyn ScalarFunction>>,
    aggregates: BTreeMap<String, Box<dyn AggregateFunction>>,
}

impl FunctionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
