use crate::core::schema::table_schema::TableSchema;
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
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    #[must_use]
    pub fn get_by_name<'a>(&'a self, schema: &'a TableSchema, name: &str) -> Option<&'a Value> {
        schema
            .column_index(name)
            .and_then(|idx| self.values.get(idx))
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
