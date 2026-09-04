use crate::{Workbook, WorkbookError};
use monumentum_db::core::row::Row;
use monumentum_db::core::schema::column::DataType;
use monumentum_db::core::value::Value;
use monumentum_db::store::storage::StorageEngine;
use std::io::Read;

impl<S: StorageEngine> Workbook<S> {
    pub fn import_csv<R: Read>(&mut self, sheet: &str, mut reader: R) -> Result<(), WorkbookError> {
        let mut content = String::new();
        let _ = reader.read_to_string(&mut content)?;

        let mut lines = content.lines();
        let header_line = lines.next().ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(
                "empty CSV input",
            ))
        })?;
        let headers = parse_csv_line(header_line);

        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet))
        })?;

        let schema_columns = table.schema().columns();
        if headers.len() != schema_columns.len() {
            return Err(WorkbookError::Db(
                monumentum_db::error::DbError::invalid_operation(
                    "CSV header column count does not match schema",
                ),
            ));
        }

        for (i, col_name) in headers.iter().enumerate() {
            let schema_col = schema_columns.get(i).ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(
                    "CSV header index out of bounds",
                ))
            })?;
            if col_name != schema_col.name() {
                return Err(WorkbookError::Db(
                    monumentum_db::error::DbError::invalid_operation(format!(
                        "CSV header '{}' does not match schema column '{}'",
                        col_name,
                        schema_col.name()
                    )),
                ));
            }
        }

        let mut rows = Vec::new();
        for (line_idx, line) in lines.enumerate() {
            let fields = parse_csv_line(line);
            if fields.len() != schema_columns.len() {
                return Err(WorkbookError::Db(
                    monumentum_db::error::DbError::invalid_operation(format!(
                        "CSV line {} has wrong number of fields",
                        line_idx.saturating_add(2)
                    )),
                ));
            }

            let mut values = Vec::with_capacity(fields.len());
            for (i, field) in fields.iter().enumerate() {
                let col = schema_columns.get(i).ok_or_else(|| {
                    WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(
                        "CSV column index out of bounds",
                    ))
                })?;
                let value = parse_csv_value(field, col.data_type())?;
                values.push(value);
            }
            rows.push(Row::new(values));
        }

        table.replace_rows(rows)?;
        Ok(())
    }

    pub fn import_json<R: Read>(
        &mut self,
        sheet: &str,
        mut reader: R,
    ) -> Result<(), WorkbookError> {
        let mut content = String::new();
        let _ = reader.read_to_string(&mut content)?;

        let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(format!(
                "invalid JSON: {e}"
            )))
        })?;
        let arr = json.as_array().ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(
                "JSON root must be an array",
            ))
        })?;

        let table = self.catalog.get_table_mut(sheet).ok_or_else(|| {
            WorkbookError::Db(monumentum_db::error::DbError::table_not_found(sheet))
        })?;
        let schema_columns = table.schema().columns();

        let mut rows = Vec::new();
        for (idx, item) in arr.iter().enumerate() {
            let obj = item.as_object().ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(format!(
                    "JSON element {idx} is not an object"
                )))
            })?;

            let mut values = Vec::with_capacity(schema_columns.len());
            for col in schema_columns {
                let key = col.name();
                let json_val = obj.get(key).ok_or_else(|| {
                    WorkbookError::Db(monumentum_db::error::DbError::invalid_operation(format!(
                        "missing field '{key}' in JSON object"
                    )))
                })?;
                let value = json_to_value(json_val, col.data_type())?;
                values.push(value);
            }
            rows.push(Row::new(values));
        }

        table.replace_rows(rows)?;
        Ok(())
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    let _ = chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else {
            match c {
                ',' => {
                    fields.push(current.clone());
                    current.clear();
                }
                '"' => in_quotes = true,
                _ => current.push(c),
            }
        }
    }
    fields.push(current);
    fields
}

fn parse_csv_value(s: &str, data_type: &DataType) -> Result<Value, WorkbookError> {
    if s.is_empty() {
        return Ok(Value::Null);
    }

    match data_type {
        DataType::Integer => {
            let i: i64 = s.parse().map_err(|e| {
                WorkbookError::Db(monumentum_db::error::DbError::type_mismatch(format!(
                    "invalid integer: {e}"
                )))
            })?;
            Ok(Value::from(i))
        }
        DataType::Float => {
            let f: f64 = s.parse().map_err(|e| {
                WorkbookError::Db(monumentum_db::error::DbError::type_mismatch(format!(
                    "invalid float: {e}"
                )))
            })?;
            Ok(Value::try_from(f)?)
        }
        DataType::Boolean => {
            let b = match s {
                "true" => true,
                "false" => false,
                "1" => true,
                "0" => false,
                _ => {
                    return Err(WorkbookError::Db(
                        monumentum_db::error::DbError::type_mismatch(format!(
                            "invalid boolean: {s}"
                        )),
                    ));
                }
            };
            Ok(Value::Boolean(b))
        }
        DataType::Text => s.strip_prefix('=').map_or_else(
            || Ok(Value::from(s.to_string())),
            |formula| Ok(Value::Formula(formula.to_string())),
        ),
        DataType::Null => Ok(Value::Null),
        DataType::Blob => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("blob import not supported"),
        )),
    }
}

fn json_to_value(json: &serde_json::Value, data_type: &DataType) -> Result<Value, WorkbookError> {
    if json.is_null() {
        return Ok(Value::Null);
    }

    match data_type {
        DataType::Integer => {
            let i = json.as_i64().ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::type_mismatch(
                    "expected integer in JSON",
                ))
            })?;
            Ok(Value::from(i))
        }
        DataType::Float => {
            let f = json.as_f64().ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::type_mismatch(
                    "expected float in JSON",
                ))
            })?;
            Ok(Value::try_from(f)?)
        }
        DataType::Boolean => {
            let b = json.as_bool().ok_or_else(|| {
                WorkbookError::Db(monumentum_db::error::DbError::type_mismatch(
                    "expected boolean in JSON",
                ))
            })?;
            Ok(Value::Boolean(b))
        }
        DataType::Text => json.as_str().map_or_else(
            || {
                Err(WorkbookError::Db(
                    monumentum_db::error::DbError::type_mismatch("expected string in JSON"),
                ))
            },
            |s| {
                s.strip_prefix('=').map_or_else(
                    || Ok(Value::from(s.to_string())),
                    |formula| Ok(Value::Formula(formula.to_string())),
                )
            },
        ),
        DataType::Null => Ok(Value::Null),
        DataType::Blob => Err(WorkbookError::Db(
            monumentum_db::error::DbError::unsupported("blob import not supported"),
        )),
    }
}
