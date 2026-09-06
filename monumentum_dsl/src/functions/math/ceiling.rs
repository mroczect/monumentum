use crate::functions::math::define_math_fn;

define_math_fn!(CeilingFunction, "ceiling", |x: f64| x.ceil());
