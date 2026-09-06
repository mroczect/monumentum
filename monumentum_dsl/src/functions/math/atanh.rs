use crate::functions::math::define_math_fn;

define_math_fn!(AtanhFunction, "atanh", |x: f64| x.atanh());
