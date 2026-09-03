use monumentum_db::core::value::Value;
use monumentum_db::types::Float;
use monumentum_query::formula::FormulaError;

pub(super) fn evaluate(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::WrongArity(
            "AVERAGE expects at least one argument".to_string(),
        ));
    }

    let mut sum: f64 = 0.0;
    for arg in args {
        match arg {
            Value::Integer(i) => {
                #[allow(clippy::cast_precision_loss)]
                {
                    sum += i.as_i64() as f64;
                }
            }
            Value::Float(f) => sum += f.as_f64(),
            Value::Null
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => {
                return Err(FormulaError::TypeMismatch(
                    "AVERAGE expects numeric values".to_string(),
                ));
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let avg = sum / args.len() as f64;
    Float::try_new(avg)
        .map(Value::Float)
        .map_err(|e| FormulaError::Eval(e.to_string()))
}
