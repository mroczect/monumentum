use monumentum_db::core::value::Value;
use monumentum_db::types::Text;

pub(super) fn evaluate(args: &[Value]) -> Value {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::Null => {}
            Value::Integer(i) => result.push_str(&i.as_i64().to_string()),
            Value::Float(f) => result.push_str(&f.as_f64().to_string()),
            Value::Text(t) => result.push_str(t.as_str()),
            Value::Blob(b) => result.push_str(&String::from_utf8_lossy(b.as_slice())),
            Value::Boolean(b) => result.push_str(if *b { "true" } else { "false" }),
            Value::Formula(s) => result.push_str(s),
            _ => {}
        }
    }
    Value::Text(Text::new(result))
}
