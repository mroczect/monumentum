use alloc::boxed::Box;
use alloc::string::String;

use crate::functions::extras::group_concat::GroupConcatAccumulator;
use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone)]
pub struct StringAggFunction {
    separator: String,
}

impl StringAggFunction {
    #[must_use]
    pub fn new(separator: impl Into<String>) -> Self {
        Self {
            separator: separator.into(),
        }
    }
}

impl Default for StringAggFunction {
    fn default() -> Self {
        Self::new(",")
    }
}

impl AggregateFunction for StringAggFunction {
    fn name(&self) -> &'static str {
        "string_agg"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        Box::new(GroupConcatAccumulator::new(self.separator.clone()))
    }
}
