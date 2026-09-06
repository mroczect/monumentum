use crate::functions::math::define_math_fn;

define_math_fn!(TruncFunction, "trunc", |x: f64| x.trunc());
