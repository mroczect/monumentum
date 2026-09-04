use crate::core::value::Value;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    #[must_use]
    pub const fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    #[must_use]
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    #[must_use]
    pub fn get<I>(&self, index: &I) -> Option<&Value>
    where
        I: crate::core::schema::column::ColumnIndex<Self>,
    {
        index.index(self).ok().and_then(|i| self.values.get(i))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
