#![allow(clippy::all)]
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

use crate::functions::ScalarFunction;
use crate::functions::json::parse_json;

#[derive(Debug, Clone, Copy)]
pub struct JsonPrettyFunction;

impl ScalarFunction for JsonPrettyFunction {
    fn name(&self) -> &'static str {
        "json_pretty"
    }

    fn call(&self, args: &[Value]) -> Result<Value, DbError> {
        let arg = args
            .first()
            .ok_or_else(|| DbError::invalid_operation("json_pretty requires one argument"))?;
        let text = arg
            .as_str()
            .ok_or_else(|| DbError::type_mismatch("argument must be text"))?;
        let parsed = parse_json(text)?;
        let pretty = pretty_print(&parsed, 0, "    ")?;
        Value::try_from(pretty)
    }
}

fn pretty_print(value: &JsonValue, indent: usize, indent_str: &str) -> Result<String, DbError> {
    let pad = indent_str.repeat(indent);
    let pad_next = indent_str.repeat(indent + 1);
    Ok(match value {
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let mut items = Vec::new();
                for (k, v) in obj {
                    items.push(format!(
                        "{pad_next}\"{}\": {}",
                        k,
                        pretty_print(v, indent + 1, indent_str)?
                    ));
                }
                format!("{{\n{}\n{pad}}}", items.join(",\n"))
            }
        }
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let mut items = Vec::new();
                for v in arr {
                    items.push(format!(
                        "{pad_next}{}",
                        pretty_print(v, indent + 1, indent_str)?
                    ));
                }
                format!("[\n{}\n{pad}]", items.join(",\n"))
            }
        }
        _ => value.to_canonical_string(),
    })
}
