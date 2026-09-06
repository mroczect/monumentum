use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone)]
pub struct GroupConcatFunction {
    separator: String,
}

impl GroupConcatFunction {
    #[must_use]
    pub fn new(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
        }
    }
}

impl Default for GroupConcatFunction {
    fn default() -> Self {
        Self::new(",")
    }
}

impl AggregateFunction for GroupConcatFunction {
    fn name(&self) -> &'static str {
        "group_concat"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(GroupConcatAccumulator::new(self.separator.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct GroupConcatAccumulator {
    separator: String,
    values: Vec<String>,
}

impl GroupConcatAccumulator {
    pub(crate) const fn new(separator: String) -> Self {
        Self {
            separator,
            values: Vec::new(),
        }
    }
}

impl Accumulator for GroupConcatAccumulator {
    fn update(&mut self, value: &Value) -> Result<(), DbError> {
        if matches!(value, Value::Null) {
            return Ok(());
        }

        if let Some(s) = value.as_str() {
            self.values.push(s.to_string());
            Ok(())
        } else {
            Err(DbError::type_mismatch("group_concat expects text values"))
        }
    }

    fn finish(self: Box<Self>) -> Result<Value, DbError> {
        let joined = self.values.join(&self.separator);
        Value::try_from(joined)
    }
}
