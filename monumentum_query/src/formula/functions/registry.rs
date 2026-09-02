use crate::formula::error::FormulaError;
use monumentum_db::core::value::Value;
use std::collections::HashMap;

pub type FunctionImpl = fn(&[Value]) -> Result<Value, FormulaError>;

#[derive(Debug, Clone, Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, FunctionImpl>,
}

impl FunctionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, func: FunctionImpl) {
        self.functions.insert(name.to_uppercase(), func);
    }

    pub fn remove(&mut self, name: &str) -> Option<FunctionImpl> {
        self.functions.remove(&name.to_uppercase())
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, FormulaError> {
        self.functions
            .get(&name.to_uppercase())
            .map(|f| f(args))
            .unwrap_or_else(|| Err(FormulaError::UnknownFunction(name.to_string())))
    }
}
