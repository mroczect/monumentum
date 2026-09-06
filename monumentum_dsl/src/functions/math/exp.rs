use crate::functions::math::define_math_fn;

define_math_fn!(ExpFunction, "exp", |x: f64| x.exp());
