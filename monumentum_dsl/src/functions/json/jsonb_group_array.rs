
#![allow(clippy::all)]
use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::json::json_group_array::JsonGroupArrayFunction;
use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct JsonbGroupArrayFunction;

impl AggregateFunction for JsonbGroupArrayFunction {
    fn name(&self) -> &'static str {
        "jsonb_group_array"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        JsonGroupArrayFunction.init()
    }
}