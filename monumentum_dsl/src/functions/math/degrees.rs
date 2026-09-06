use crate::functions::math::define_math_fn;

define_math_fn!(DegreesFunction, "degrees", |x: f64| x.to_degrees());
