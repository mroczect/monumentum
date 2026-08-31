use crate::core::value::Value;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    #[must_use]
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
