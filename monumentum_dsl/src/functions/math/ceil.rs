use crate::functions::math::define_math_fn;

define_math_fn!(CeilFunction, "ceil", |x: f64| x.ceil());
