use monumentum_db::core::value::Value;
use monumentum_db::types::{Float, Integer};
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "SUM requires at least one argument".to_string(),
        ));
    }

    let mut sum_int: i64 = 0;
    let mut sum_float: f64 = 0.0;
    let mut has_float = false;

    for arg in args {
        match arg {
            Value::Integer(i) => {
                if has_float {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        sum_float += i.as_i64() as f64;
                    }
                } else {
                    sum_int = sum_int
                        .checked_add(i.as_i64())
                        .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    has_float = true;
                    #[allow(clippy::cast_precision_loss)]
                    {
                        sum_float = sum_int as f64;
                    }
                }
                sum_float += f.as_f64();
                if !sum_float.is_finite() {
                    return Err(FormulaError::Eval("float overflow".to_string()));
                }
            }
            Value::Null
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => {
                return Err(FormulaError::TypeMismatch(
                    "SUM expects numeric values".to_string(),
                ));
            }
        }
    }

    if has_float {
        Float::try_new(sum_float)
            .map(Value::Float)
            .map_err(|e| FormulaError::Eval(e.to_string()))
    } else {
        Ok(Value::Integer(Integer::new(sum_int)))
    }
}
