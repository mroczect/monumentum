use crate::functions::math::define_math_fn;

define_math_fn!(LnFunction, "ln", |x: f64| x.ln());
