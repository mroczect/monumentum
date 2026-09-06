use crate::functions::math::define_math_fn;

define_math_fn!(Log10Function, "log10", |x: f64| x.log10());
