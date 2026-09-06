#![allow(clippy::all)]
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

pub mod json;
pub use json::*;

pub mod jsonb;
pub use jsonb::*;

pub mod json_array;
pub use json_array::*;

pub mod jsonb_array;
pub use jsonb_array::*;

pub mod json_array_insert;
pub use json_array_insert::*;

pub mod jsonb_array_insert;
pub use jsonb_array_insert::*;

pub mod json_array_length;
pub use json_array_length::*;

pub mod json_error_position;
pub use json_error_position::*;

pub mod json_extract;
pub use json_extract::*;

pub mod jsonb_extract;
pub use jsonb_extract::*;

pub mod json_arrow;
pub use json_arrow::*;

pub mod json_arrow_double;
pub use json_arrow_double::*;

pub mod json_insert;
pub use json_insert::*;

pub mod jsonb_insert;
pub use jsonb_insert::*;

pub mod json_object;
pub use json_object::*;

pub mod jsonb_object;
pub use jsonb_object::*;

pub mod json_patch;
pub use json_patch::*;

pub mod jsonb_patch;
pub use jsonb_patch::*;

pub mod json_pretty;
pub use json_pretty::*;

pub mod json_quote;
pub use json_quote::*;

pub mod json_remove;
pub use json_remove::*;

pub mod jsonb_remove;
pub use jsonb_remove::*;

pub mod json_replace;
pub use json_replace::*;

pub mod jsonb_replace;
pub use jsonb_replace::*;

pub mod json_set;
pub use json_set::*;

pub mod jsonb_set;
pub use jsonb_set::*;

pub mod json_type;
pub use json_type::*;

pub mod json_valid;
pub use json_valid::*;

pub mod json_group_array;
pub use json_group_array::*;

pub mod jsonb_group_array;
pub use jsonb_group_array::*;

pub mod json_group_object;
pub use json_group_object::*;

pub mod jsonb_group_object;
pub use jsonb_group_object::*;

pub mod json_each;
pub use json_each::*;

pub mod jsonb_each;
pub use jsonb_each::*;

pub mod json_tree;
pub use json_tree::*;

pub mod jsonb_tree;
pub use jsonb_tree::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub(crate) fn to_canonical_string(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(true) => "true".to_string(),
            JsonValue::Bool(false) => "false".to_string(),
            JsonValue::Int(i) => i.to_string(),
            JsonValue::Float(f) => {
                if f.is_nan() {
                    "null".to_string()
                } else {
                    f.to_string()
                }
            }
            JsonValue::Str(s) => json_escape(s),
            JsonValue::Array(arr) => {
                let mut out = String::from("[");
                for (idx, item) in arr.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    out.push_str(&item.to_canonical_string());
                }
                out.push(']');
                out
            }
            JsonValue::Object(obj) => {
                let mut out = String::from("{");
                for (idx, (key, val)) in obj.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    out.push_str(&json_escape(key));
                    out.push(':');
                    out.push_str(&val.to_canonical_string());
                }
                out.push('}');
                out
            }
        }
    }

    pub(crate) fn to_sql_value(&self) -> Result<Value, DbError> {
        match self {
            JsonValue::Null => Ok(Value::Null),
            JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
            JsonValue::Int(i) => Ok(Value::from(*i)),
            JsonValue::Float(f) => Value::try_from(*f),
            JsonValue::Str(s) => Value::try_from(s.clone()),
            JsonValue::Array(_) | JsonValue::Object(_) => {
                Value::try_from(self.to_canonical_string())
            }
        }
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub(crate) fn parse_json(input: &str) -> Result<JsonValue, DbError> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    parser.skip_ws();
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos < parser.bytes.len() {
        return Err(DbError::invalid_operation("trailing characters after JSON"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\n' | b'\r' | b'\t' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn expect(&mut self, b: u8) -> Result<(), DbError> {
        if self.next() == Some(b) {
            Ok(())
        } else {
            Err(DbError::invalid_operation("unexpected character"))
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, DbError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') | Some(b'\'') => Ok(JsonValue::Str(self.parse_string()?)),
            Some(b't') | Some(b'f') => self.parse_bool(),
            Some(b'n') => self.parse_null(),
            Some(_) => self.parse_number(),
            None => Err(DbError::invalid_operation("unexpected end of JSON")),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, DbError> {
        self.expect(b'{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.next();
            return Ok(JsonValue::Object(pairs));
        }
        loop {
            self.skip_ws();
            let key = match self.peek() {
                Some(b'"') | Some(b'\'') => self.parse_string()?,
                _ => return Err(DbError::invalid_operation("object key must be string")),
            };
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip_ws();
            match self.next() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(DbError::invalid_operation("expected ',' or '}'")),
            }
        }
        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, DbError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.next();
            return Ok(JsonValue::Array(items));
        }
        loop {
            self.skip_ws();
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.next() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(DbError::invalid_operation("expected ',' or ']'")),
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, DbError> {
        let quote = self
            .next()
            .ok_or_else(|| DbError::invalid_operation("unexpected end"))?;
        let mut out = String::new();
        loop {
            match self.next() {
                Some(b) if b == quote => break,
                Some(b'\\') => {
                    let esc = self
                        .next()
                        .ok_or_else(|| DbError::invalid_operation("bad escape"))?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let digit = self.next().ok_or_else(|| {
                                    DbError::invalid_operation("bad unicode escape")
                                })?;
                                code = code * 16
                                    + (digit as char).to_digit(16).ok_or_else(|| {
                                        DbError::invalid_operation("bad unicode escape")
                                    })?;
                            }
                            if let Some(c) = char::from_u32(code) {
                                out.push(c);
                            } else {
                                return Err(DbError::invalid_operation(
                                    "invalid unicode codepoint",
                                ));
                            }
                        }
                        _ => return Err(DbError::invalid_operation("invalid escape")),
                    }
                }
                Some(b) => out.push(b as char),
                None => return Err(DbError::invalid_operation("unterminated string")),
            }
        }
        Ok(out)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, DbError> {
        if self.bytes[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.bytes[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(DbError::invalid_operation("invalid boolean"))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, DbError> {
        if self.bytes[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(DbError::invalid_operation("invalid null"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, DbError> {
        let start = self.pos;
        while self.pos < self.bytes.len()
            && matches!(
                self.bytes[self.pos],
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E'
            )
        {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| DbError::invalid_operation("invalid UTF-8 in number"))?;
        if let Ok(i) = s.parse::<i64>() {
            Ok(JsonValue::Int(i))
        } else if let Ok(f) = s.parse::<f64>() {
            Ok(JsonValue::Float(f))
        } else {
            Err(DbError::invalid_operation("invalid number"))
        }
    }
}

pub(crate) fn extract_path(root: &JsonValue, path: &str) -> Result<Option<JsonValue>, DbError> {
    if !path.starts_with('$') {
        return Err(DbError::invalid_operation("path must start with '$'"));
    }
    let mut current = Some(root.clone());
    let mut rest = &path[1..];
    while !rest.is_empty() {
        match rest.chars().next().unwrap() {
            '.' => {
                rest = &rest[1..];
                let end = rest.find(|c| c == '.' || c == '[').unwrap_or(rest.len());
                let key = &rest[..end];
                current = match current {
                    Some(JsonValue::Object(obj)) => {
                        obj.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
                    }
                    _ => None,
                };
                rest = &rest[end..];
            }
            '[' => {
                rest = &rest[1..];
                let end = rest
                    .find(']')
                    .ok_or_else(|| DbError::invalid_operation("unterminated array index"))?;
                let idx_str = &rest[..end];
                let idx = if idx_str == "#" {
                    match current {
                        Some(JsonValue::Array(arr)) => arr.len().saturating_sub(1),
                        _ => return Ok(None),
                    }
                } else if let Some(stripped) = idx_str.strip_prefix("#-") {
                    if let Ok(n) = stripped.parse::<usize>() {
                        match current {
                            Some(JsonValue::Array(arr)) => arr.len().saturating_sub(n),
                            _ => return Ok(None),
                        }
                    } else {
                        return Err(DbError::invalid_operation("invalid array index"));
                    }
                } else {
                    match idx_str.parse::<usize>() {
                        Ok(n) => n,
                        Err(_) => return Err(DbError::invalid_operation("invalid array index")),
                    }
                };
                current = match current {
                    Some(JsonValue::Array(arr)) => arr.get(idx).cloned(),
                    _ => None,
                };
                rest = &rest[end + 1..];
            }
            _ => return Err(DbError::invalid_operation("invalid path syntax")),
        }
    }
    Ok(current)
}

pub(crate) fn json_from_value(value: &Value) -> Result<JsonValue, DbError> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
        Value::Integer(i) => Ok(JsonValue::Int(i.as_i64())),
        Value::Float(f) => Ok(JsonValue::Float(f.as_f64())),
        Value::Text(t) => parse_json(t.as_str()),
        Value::Blob(_) => Err(DbError::type_mismatch("BLOB not supported as JSON")),
    }
}
