mod and;
mod average;
mod concat;
mod if_fn;
mod len;
mod lower;
mod max;
mod min;
mod not;
mod or;
mod sum;
mod trim;
mod upper;

use monumentum_query::formula::FunctionRegistry;

pub fn register_all(registry: &mut FunctionRegistry) {
    registry.register("SUM", sum::evaluate);
    registry.register("AVERAGE", average::evaluate);
    registry.register("AVG", average::evaluate);
    registry.register("MIN", min::evaluate);
    registry.register("MAX", max::evaluate);
    registry.register("IF", if_fn::evaluate);
    registry.register("AND", and::evaluate);
    registry.register("OR", or::evaluate);
    registry.register("NOT", not::evaluate);
    registry.register("CONCAT", |args| Ok(concat::evaluate(args)));
    registry.register("CONCATENATE", |args| Ok(concat::evaluate(args)));
    registry.register("TRIM", trim::evaluate);
    registry.register("UPPER", upper::evaluate);
    registry.register("LOWER", lower::evaluate);
    registry.register("LEN", len::evaluate);
}
