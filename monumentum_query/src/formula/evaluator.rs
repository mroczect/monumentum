use crate::coordinates::parse_cell_ref;
use crate::formula::ast::{BinaryOp, Expr, UnaryOp};
use crate::formula::context::FormulaContext;
use crate::formula::error::FormulaError;
use crate::formula::functions::FunctionRegistry;
use monumentum_db::core::value::Value;
use monumentum_db::types::{Float, Integer, Text};

const MAX_RANGE_CELLS: usize = 100_000;

pub fn evaluate(
    expr: &Expr,
    ctx: &dyn FormulaContext,
    registry: &FunctionRegistry,
) -> Result<Value, FormulaError> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::CellRef(s) => {
            let cell =
                parse_cell_ref(s).map_err(|e| FormulaError::InvalidReference(e.to_string()))?;
            ctx.get_cell_value(&cell)
        }
        Expr::Range(_) => Err(FormulaError::Eval(
            "range not allowed in scalar context".to_string(),
        )),
        Expr::UnaryOp(op, operand) => {
            let val = evaluate(operand, ctx, registry)?;
            apply_unary(*op, val)
        }
        Expr::BinaryOp(op, left, right) => {
            let l = evaluate(left, ctx, registry)?;
            let r = evaluate(right, ctx, registry)?;
            apply_binary(*op, l, r)
        }
        Expr::FunctionCall(name, args) => {
            let mut arg_values = Vec::new();
            for arg in args {
                match arg {
                    Expr::Range(range) => {
                        let cell_count = (range.end.row - range.start.row + 1) as usize
                            * (range.end.col - range.start.col + 1) as usize;
                        if cell_count > MAX_RANGE_CELLS {
                            return Err(FormulaError::Eval("range too large".to_string()));
                        }
                        for cell in range.iter() {
                            let v = ctx.get_cell_value(&cell)?;
                            arg_values.push(v);
                        }
                    }
                    _ => {
                        let v = evaluate(arg, ctx, registry)?;
                        arg_values.push(v);
                    }
                }
            }
            registry.call(name, &arg_values)
        }
    }
}

fn apply_unary(op: UnaryOp, val: Value) -> Result<Value, FormulaError> {
    match op {
        UnaryOp::Neg => match val {
            Value::Integer(i) => {
                let n = i.as_i64().checked_neg().ok_or_else(|| {
                    FormulaError::Eval("integer overflow on negation".to_string())
                })?;
                Ok(Value::Integer(Integer::new(n)))
            }
            Value::Float(f) => {
                let n = -f.as_f64();
                Float::try_new(n)
                    .map(Value::Float)
                    .map_err(|e| FormulaError::Eval(e.to_string()))
            }
            _ => Err(FormulaError::TypeMismatch(
                "cannot negate non-numeric value".to_string(),
            )),
        },
        UnaryOp::Not => match val {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(FormulaError::TypeMismatch(
                "cannot apply logical NOT to non-boolean value".to_string(),
            )),
        },
    }
}

fn apply_binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, FormulaError> {
    use BinaryOp::*;
    match op {
        Add => add_values(left, right),
        Sub => sub_values(left, right),
        Mul => mul_values(left, right),
        Div => div_values(left, right),
        Mod => mod_values(left, right),
        Pow => pow_values(left, right),
        Eq => Ok(Value::Boolean(left == right)),
        NotEq => Ok(Value::Boolean(left != right)),
        Lt | Lte | Gt | Gte => {
            let ord = left.partial_cmp(&right).ok_or_else(|| {
                FormulaError::TypeMismatch("values are not comparable".to_string())
            })?;
            let result = match op {
                Lt => ord == std::cmp::Ordering::Less,
                Lte => ord != std::cmp::Ordering::Greater,
                Gt => ord == std::cmp::Ordering::Greater,
                Gte => ord != std::cmp::Ordering::Less,
                _ => unreachable!(),
            };
            Ok(Value::Boolean(result))
        }
        And => match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
            _ => Err(FormulaError::TypeMismatch(
                "AND requires boolean operands".to_string(),
            )),
        },
        Or => match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
            _ => Err(FormulaError::TypeMismatch(
                "OR requires boolean operands".to_string(),
            )),
        },
    }
}

fn add_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            let sum = a
                .as_i64()
                .checked_add(b.as_i64())
                .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
            Ok(Value::Integer(Integer::new(sum)))
        }
        (Value::Float(a), Value::Float(b)) => {
            let sum = a.as_f64() + b.as_f64();
            Float::try_new(sum)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            let sum = a.as_i64() as f64 + b.as_f64();
            Float::try_new(sum)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Text(a), Value::Text(b)) => {
            let mut s = a.as_str().to_string();
            s.push_str(b.as_str());
            Ok(Value::Text(Text::new(s)))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot add values of these types".to_string(),
        )),
    }
}

fn sub_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            let diff = a
                .as_i64()
                .checked_sub(b.as_i64())
                .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
            Ok(Value::Integer(Integer::new(diff)))
        }
        (Value::Float(a), Value::Float(b)) => {
            let diff = a.as_f64() - b.as_f64();
            Float::try_new(diff)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            let diff = a.as_i64() as f64 - b.as_f64();
            Float::try_new(diff)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot subtract values of these types".to_string(),
        )),
    }
}

fn mul_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            let prod = a
                .as_i64()
                .checked_mul(b.as_i64())
                .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
            Ok(Value::Integer(Integer::new(prod)))
        }
        (Value::Float(a), Value::Float(b)) => {
            let prod = a.as_f64() * b.as_f64();
            Float::try_new(prod)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            let prod = a.as_i64() as f64 * b.as_f64();
            Float::try_new(prod)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot multiply values of these types".to_string(),
        )),
    }
}

fn div_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(_), Value::Integer(b)) if b.as_i64() == 0 => {
            Err(FormulaError::DivisionByZero)
        }
        (Value::Integer(a), Value::Integer(b)) => {
            let quo = a
                .as_i64()
                .checked_div(b.as_i64())
                .ok_or_else(|| FormulaError::Eval("integer division error".to_string()))?;
            Ok(Value::Integer(Integer::new(quo)))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b.as_f64() == 0.0 {
                return Err(FormulaError::DivisionByZero);
            }
            let quo = a.as_f64() / b.as_f64();
            Float::try_new(quo)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            if b.as_f64() == 0.0 {
                return Err(FormulaError::DivisionByZero);
            }
            let quo = a.as_i64() as f64 / b.as_f64();
            Float::try_new(quo)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot divide values of these types".to_string(),
        )),
    }
}

fn mod_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(_), Value::Integer(b)) if b.as_i64() == 0 => {
            Err(FormulaError::DivisionByZero)
        }
        (Value::Integer(a), Value::Integer(b)) => {
            let rem = a
                .as_i64()
                .checked_rem(b.as_i64())
                .ok_or_else(|| FormulaError::Eval("integer modulo error".to_string()))?;
            Ok(Value::Integer(Integer::new(rem)))
        }
        (Value::Float(a), Value::Float(b)) => {
            if b.as_f64() == 0.0 {
                return Err(FormulaError::DivisionByZero);
            }
            let rem = a.as_f64() % b.as_f64();
            Float::try_new(rem)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            if b.as_f64() == 0.0 {
                return Err(FormulaError::DivisionByZero);
            }
            let rem = a.as_i64() as f64 % b.as_f64();
            Float::try_new(rem)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot apply modulo to values of these types".to_string(),
        )),
    }
}

fn pow_values(l: Value, r: Value) -> Result<Value, FormulaError> {
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => {
            if b.as_i64() >= 0 {
                let base = a.as_i64();
                let exp = b.as_i64() as u32;
                let result = base
                    .checked_pow(exp)
                    .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
                Ok(Value::Integer(Integer::new(result)))
            } else {
                let base = a.as_i64() as f64;
                let exp = b.as_i64() as f64;
                let result = base.powf(exp);
                if !result.is_finite() {
                    return Err(FormulaError::Eval("float result is not finite".to_string()));
                }
                Float::try_new(result)
                    .map(Value::Float)
                    .map_err(|e| FormulaError::Eval(e.to_string()))
            }
        }
        (Value::Float(a), Value::Float(b)) => {
            let result = a.as_f64().powf(b.as_f64());
            if !result.is_finite() {
                return Err(FormulaError::Eval("float result is not finite".to_string()));
            }
            Float::try_new(result)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            let result = (a.as_i64() as f64).powf(b.as_f64());
            if !result.is_finite() {
                return Err(FormulaError::Eval("float result is not finite".to_string()));
            }
            Float::try_new(result)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => Err(FormulaError::TypeMismatch(
            "cannot raise these types to a power".to_string(),
        )),
    }
}
