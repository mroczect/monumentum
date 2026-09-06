#![allow(clippy::all)]
use alloc::boxed::Box;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::json::json_group_object::JsonGroupObjectFunction;
use crate::functions::{Accumulator, AggregateFunction};

#[derive(Debug, Clone, Copy)]
pub struct JsonbGroupObjectFunction;

impl AggregateFunction for JsonbGroupObjectFunction {
    fn name(&self) -> &'static str {
        "jsonb_group_object"
    }

    fn init(&self) -> Box<dyn Accumulator> {
        JsonGroupObjectFunction.init()
    }
}
