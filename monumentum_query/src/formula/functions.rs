use crate::formula::error::FormulaError;
use monumentum_db::core::value::Value;
use monumentum_db::types::{Float, Integer, Text};

pub fn call_function(name: &str, args: &[Value]) -> Result<Value, FormulaError> {
    let name_upper = name.to_uppercase();
    match name_upper.as_str() {
        "SUM" => func_sum(args),
        "AVERAGE" | "AVG" => func_average(args),
        "MIN" => func_min(args),
        "MAX" => func_max(args),
        "IF" => func_if(args),
        "AND" => func_and(args),
        "OR" => func_or(args),
        "NOT" => func_not(args),
        "CONCAT" | "CONCATENATE" => func_concat(args),
        "TRIM" => func_trim(args),
        "UPPER" => func_upper(args),
        "LOWER" => func_lower(args),
        "LEN" => func_len(args),
        _ => Err(FormulaError::UnknownFunction(name.to_string())),
    }
}

fn func_sum(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "SUM needs at least one value".to_string(),
        ));
    }
    let mut sum_int: i64 = 0;
    let mut sum_float: f64 = 0.0;
    let mut has_float = false;
    for val in args {
        match val {
            Value::Integer(i) => {
                if has_float {
                    sum_float += i.as_i64() as f64;
                } else {
                    sum_int = sum_int
                        .checked_add(i.as_i64())
                        .ok_or_else(|| FormulaError::Eval("integer overflow".to_string()))?;
                }
            }
            Value::Float(f) => {
                if !has_float {
                    has_float = true;
                    sum_float = sum_int as f64;
                }
                sum_float += f.as_f64();
                if !sum_float.is_finite() {
                    return Err(FormulaError::Eval("float overflow".to_string()));
                }
            }
            _ => {
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

fn func_average(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "AVERAGE needs at least one value".to_string(),
        ));
    }
    let sum = func_sum(args)?;
    let count = args.len() as f64;
    match sum {
        Value::Integer(i) => {
            let avg = i.as_i64() as f64 / count;
            Float::try_new(avg)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        Value::Float(f) => {
            let avg = f.as_f64() / count;
            Float::try_new(avg)
                .map(Value::Float)
                .map_err(|e| FormulaError::Eval(e.to_string()))
        }
        _ => unreachable!(),
    }
}

fn func_min(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "MIN needs at least one value".to_string(),
        ));
    }
    let mut min_val = args[0].clone();
    for val in &args[1..] {
        if val
            .partial_cmp(&min_val)
            .unwrap_or(std::cmp::Ordering::Greater)
            == std::cmp::Ordering::Less
        {
            min_val = val.clone();
        }
    }
    Ok(min_val)
}

fn func_max(args: &[Value]) -> Result<Value, FormulaError> {
    if args.is_empty() {
        return Err(FormulaError::Eval(
            "MAX needs at least one value".to_string(),
        ));
    }
    let mut max_val = args[0].clone();
    for val in &args[1..] {
        if val
            .partial_cmp(&max_val)
            .unwrap_or(std::cmp::Ordering::Less)
            == std::cmp::Ordering::Greater
        {
            max_val = val.clone();
        }
    }
    Ok(max_val)
}

fn func_if(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 3 {
        return Err(FormulaError::WrongArity(
            "IF expects 3 arguments".to_string(),
        ));
    }
    let condition = match &args[0] {
        Value::Boolean(b) => *b,
        _ => {
            return Err(FormulaError::TypeMismatch(
                "IF condition must be boolean".to_string(),
            ));
        }
    };
    if condition {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

fn func_and(args: &[Value]) -> Result<Value, FormulaError> {
    for val in args {
        match val {
            Value::Boolean(b) => {
                if !*b {
                    return Ok(Value::Boolean(false));
                }
            }
            _ => {
                return Err(FormulaError::TypeMismatch(
                    "AND expects boolean arguments".to_string(),
                ));
            }
        }
    }
    Ok(Value::Boolean(true))
}

fn func_or(args: &[Value]) -> Result<Value, FormulaError> {
    for val in args {
        match val {
            Value::Boolean(b) => {
                if *b {
                    return Ok(Value::Boolean(true));
                }
            }
            _ => {
                return Err(FormulaError::TypeMismatch(
                    "OR expects boolean arguments".to_string(),
                ));
            }
        }
    }
    Ok(Value::Boolean(false))
}

fn func_not(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity(
            "NOT expects 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Boolean(b) => Ok(Value::Boolean(!*b)),
        _ => Err(FormulaError::TypeMismatch(
            "NOT expects boolean argument".to_string(),
        )),
    }
}

fn func_concat(args: &[Value]) -> Result<Value, FormulaError> {
    let mut result = String::new();
    for val in args {
        match val {
            Value::Text(t) => result.push_str(t.as_str()),
            Value::Integer(i) => result.push_str(&i.to_string()),
            Value::Float(f) => result.push_str(&f.to_string()),
            Value::Boolean(b) => result.push_str(&b.to_string()),
            Value::Null => {}
            _ => {
                return Err(FormulaError::TypeMismatch(
                    "CONCAT cannot handle blob values".to_string(),
                ));
            }
        }
    }
    Ok(Value::Text(Text::new(result)))
}

fn func_trim(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity(
            "TRIM expects 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Text(t) => Ok(Value::Text(Text::new(t.as_str().trim().to_string()))),
        _ => Err(FormulaError::TypeMismatch(
            "TRIM expects text argument".to_string(),
        )),
    }
}

fn func_upper(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity(
            "UPPER expects 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Text(t) => Ok(Value::Text(Text::new(t.as_str().to_uppercase()))),
        _ => Err(FormulaError::TypeMismatch(
            "UPPER expects text argument".to_string(),
        )),
    }
}

fn func_lower(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity(
            "LOWER expects 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Text(t) => Ok(Value::Text(Text::new(t.as_str().to_lowercase()))),
        _ => Err(FormulaError::TypeMismatch(
            "LOWER expects text argument".to_string(),
        )),
    }
}

fn func_len(args: &[Value]) -> Result<Value, FormulaError> {
    if args.len() != 1 {
        return Err(FormulaError::WrongArity(
            "LEN expects 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Text(t) => Ok(Value::Integer(Integer::new(t.as_str().len() as i64))),
        _ => Err(FormulaError::TypeMismatch(
            "LEN expects text argument".to_string(),
        )),
    }
}
