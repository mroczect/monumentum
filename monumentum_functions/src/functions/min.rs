use core::cmp::Ordering;
use monumentum_db::core::value::Value;
use monumentum_db::types::Float;
use monumentum_query::formula::FormulaError;

#[allow(clippy::cast_precision_loss, clippy::wildcard_enum_match_arm)]
pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "MIN requires at least one argument".to_string(),
        ));
    }

    let mut min_val: Option<Value> = None;
    let mut all_integer = true;

    for arg in args {
        match arg {
            Value::Integer(_) => {}
            Value::Float(_) => all_integer = false,
            Value::Null
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => {
                return Err(FormulaError::TypeMismatch(
                    "MIN expects numeric values".to_string(),
                ));
            }
        }

        if let Some(ref cur) = min_val {
            let ordering = compare_numeric(arg, cur)?;
            if ordering == Ordering::Less {
                min_val = Some(arg.clone());
            }
        } else {
            min_val = Some(arg.clone());
        }
    }

    let value = min_val.ok_or_else(|| FormulaError::Eval("MIN failed".to_string()))?;
    if all_integer {
        match value {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Float(f) => Ok(Value::Float(f)),
            _ => Err(FormulaError::Eval(
                "unexpected non-numeric value".to_string(),
            )),
        }
    } else {
        match value {
            Value::Integer(i) => Float::try_new(i.as_i64() as f64)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string())),
            Value::Float(f) => Ok(Value::Float(f)),
            _ => Err(FormulaError::Eval(
                "unexpected non-numeric value".to_string(),
            )),
        }
    }
}

#[allow(clippy::cast_precision_loss, clippy::wildcard_enum_match_arm)]
fn compare_numeric(a: &Value, b: &Value) -> Result<Ordering, FormulaError> {
    let a_num = match a {
        Value::Integer(i) => i.as_i64() as f64,
        Value::Float(f) => f.as_f64(),
        _ => return Err(FormulaError::TypeMismatch("not numeric".to_string())),
    };
    let b_num = match b {
        Value::Integer(i) => i.as_i64() as f64,
        Value::Float(f) => f.as_f64(),
        _ => return Err(FormulaError::TypeMismatch("not numeric".to_string())),
    };
    Ok(a_num.partial_cmp(&b_num).unwrap_or(Ordering::Equal))
}
