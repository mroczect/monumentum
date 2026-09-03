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
    pub fn get<I>(&self, index: I) -> Option<&Value>
    where
        I: crate::core::schema::column::ColumnIndex<Self>,
    {
        index.index(self).ok().and_then(|i| self.values.get(i))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[must_use]
    pub fn values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.values.get_mut(index)
    }
}
