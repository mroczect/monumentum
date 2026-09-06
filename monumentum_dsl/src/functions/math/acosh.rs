use crate::functions::math::define_math_fn;

define_math_fn!(AcoshFunction, "acosh", |x: f64| x.acosh());
