use crate::functions::math::define_math_fn;

define_math_fn!(AtanFunction, "atan", |x: f64| x.atan());
