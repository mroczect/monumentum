use crate::{Workbook, WorkbookError};
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
use std::io::Write;

impl<S: StorageEngine> Workbook<S> {
    pub fn export_csv<W: Write>(&self, sheet: &str, mut writer: W) -> Result<(), WorkbookError> {
        let table = self.sheet(sheet)?;
        let columns = table.schema().columns();

        let header: Vec<String> = columns.iter().map(|c| c.name().to_string()).collect();
        write_csv_line(&mut writer, &header)?;

        for row in table.rows() {
            let mut line = Vec::with_capacity(columns.len());
            for (i, _) in columns.iter().enumerate() {
                let value = row.get(i).ok_or(WorkbookError::InvalidReference)?;
                let s = value_to_csv_string(value)?;
                line.push(s);
            }
            write_csv_line(&mut writer, &line)?;
        }

        Ok(())
    }

    pub fn export_json<W: Write>(&self, sheet: &str, mut writer: W) -> Result<(), WorkbookError> {
        let table = self.sheet(sheet)?;
        let columns = table.schema().columns();

        let mut arr = Vec::new();
        for row in table.rows() {
            let mut obj = serde_json::Map::new();
            for (i, col) in columns.iter().enumerate() {
                let value = row.get(i).ok_or(WorkbookError::InvalidReference)?;
                let json_val = value_to_json(value)?;
                let _ = obj.insert(col.name().to_string(), json_val);
            }
            arr.push(serde_json::Value::Object(obj));
        }

        let json = serde_json::Value::Array(arr);
        writer.write_all(json.to_string().as_bytes())?;
        Ok(())
    }
}

fn write_csv_line<W: Write>(writer: &mut W, fields: &[String]) -> Result<(), WorkbookError> {
    let mut out = String::new();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&escape_csv_field(field));
    }
    out.push('\n');
    writer.write_all(out.as_bytes())?;
    Ok(())
}

fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn value_to_csv_string(value: &Value) -> Result<String, WorkbookError> {
    match value {
        Value::Null => Ok(String::new()),
        Value::Integer(i) => Ok(i.as_i64().to_string()),
        Value::Float(f) => Ok(f.as_f64().to_string()),
        Value::Text(t) => Ok(t.as_str().to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Formula(s) => Ok(format!("={s}")),
        Value::Blob(_) => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("blob not supported in CSV"),
        )),
        _ => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("unknown type not supported in CSV"),
        )),
    }
}

fn value_to_json(value: &Value) -> Result<serde_json::Value, WorkbookError> {
    match value {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Integer(i) => Ok(serde_json::Value::from(i.as_i64())),
        Value::Float(f) => Ok(serde_json::Value::from(f.as_f64())),
        Value::Text(t) => Ok(serde_json::Value::String(t.as_str().to_string())),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Formula(s) => Ok(serde_json::Value::String(format!("={s}"))),
        Value::Blob(_) => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("blob not supported in JSON"),
        )),
        _ => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("unknown type not supported in JSON"),
        )),
    }
}
