use crate::functions::math::define_math_fn;

define_math_fn!(SinhFunction, "sinh", |x: f64| x.sinh());
