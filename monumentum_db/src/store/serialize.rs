use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::{ColumnDef, ComparisonOp, DataType};
use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use crate::core::value::Value;
use crate::error::DbError;
use crate::types::{Blob, Float, Integer, Text};
use std::io::{Cursor, Read};

const TAG_NULL: u8 = 0;
const TAG_INTEGER: u8 = 1;
const TAG_FLOAT: u8 = 2;
const TAG_TEXT: u8 = 3;
const TAG_BLOB: u8 = 4;

const FORMAT_VERSION: u32 = 1;
const MAX_READ_BYTES: usize = 64 * 1024 * 1024;

fn write_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    write_u64(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, DbError> {
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32, DbError> {
    let mut b = [0u8; 4];
    cursor.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64, DbError> {
    let mut b = [0u8; 8];
    cursor.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

fn read_bytes(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>, DbError> {
    let len = read_u64(cursor)? as usize;
    if len > MAX_READ_BYTES {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("declared length {} exceeds maximum allowed", len),
        )));
    }
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}

fn encode_data_type(dt: &DataType) -> u8 {
    match dt {
        DataType::Null => 0,
        DataType::Integer => 1,
        DataType::Float => 2,
        DataType::Text => 3,
        DataType::Blob => 4,
    }
}

fn decode_data_type(v: u8) -> Result<DataType, DbError> {
    match v {
        0 => Ok(DataType::Null),
        1 => Ok(DataType::Integer),
        2 => Ok(DataType::Float),
        3 => Ok(DataType::Text),
        4 => Ok(DataType::Blob),
        _ => Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid data type tag",
        ))),
    }
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    match value {
        Value::Null => {
            write_u8(&mut buf, TAG_NULL);
        }
        Value::Integer(i) => {
            write_u8(&mut buf, TAG_INTEGER);
            write_u64(&mut buf, i.as_i64() as u64);
        }
        Value::Float(f) => {
            write_u8(&mut buf, TAG_FLOAT);
            buf.extend_from_slice(&f.as_f64().to_le_bytes());
        }
        Value::Text(t) => {
            write_u8(&mut buf, TAG_TEXT);
            write_bytes(&mut buf, t.as_bytes());
        }
        Value::Blob(b) => {
            write_u8(&mut buf, TAG_BLOB);
            write_bytes(&mut buf, b.as_slice());
        }
    }
    buf
}

fn decode_value(cursor: &mut Cursor<&[u8]>) -> Result<Value, DbError> {
    let tag = read_u8(cursor)?;
    match tag {
        TAG_NULL => Ok(Value::Null),
        TAG_INTEGER => {
            let raw = read_u64(cursor)?;
            Ok(Value::Integer(Integer::new(raw as i64)))
        }
        TAG_FLOAT => {
            let mut b = [0u8; 8];
            cursor.read_exact(&mut b)?;
            let f = f64::from_le_bytes(b);
            Float::try_new(f).map(Value::Float)
        }
        TAG_TEXT => {
            let bytes = read_bytes(cursor)?;
            let s = String::from_utf8(bytes).map_err(|e| {
                DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            })?;
            Ok(Value::Text(Text::new(s)))
        }
        TAG_BLOB => {
            let bytes = read_bytes(cursor)?;
            Ok(Value::Blob(Blob::new(bytes)))
        }
        _ => Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid value tag",
        ))),
    }
}

pub fn encode_column_def(col: &ColumnDef) -> Vec<u8> {
    let mut buf = Vec::new();
    write_bytes(&mut buf, col.name().as_bytes());
    write_u8(&mut buf, encode_data_type(col.data_type()));
    write_u8(&mut buf, col.is_nullable() as u8);
    write_u8(&mut buf, col.is_primary_key() as u8);
    write_u8(&mut buf, col.is_unique() as u8);

    match col.default_value() {
        Some(v) => {
            write_u8(&mut buf, 1);
            let val_bytes = encode_value(v);
            write_bytes(&mut buf, &val_bytes);
        }
        None => write_u8(&mut buf, 0),
    }

    match col.check_constraint() {
        Some(cc) => {
            write_u8(&mut buf, 1);
            write_bytes(&mut buf, cc.column.as_bytes());
            let op_tag = match cc.op {
                ComparisonOp::Eq => 0,
                ComparisonOp::NotEq => 1,
                ComparisonOp::Lt => 2,
                ComparisonOp::Lte => 3,
                ComparisonOp::Gt => 4,
                ComparisonOp::Gte => 5,
            };
            write_u8(&mut buf, op_tag);
            let val_bytes = encode_value(&cc.value);
            write_bytes(&mut buf, &val_bytes);
        }
        None => write_u8(&mut buf, 0),
    }

    match col.foreign_key() {
        Some(fk) => {
            write_u8(&mut buf, 1);
            write_bytes(&mut buf, fk.table.as_bytes());
            write_bytes(&mut buf, fk.column.as_bytes());
        }
        None => write_u8(&mut buf, 0),
    }

    buf
}

pub fn decode_column_def(cursor: &mut Cursor<&[u8]>) -> Result<ColumnDef, DbError> {
    let name_bytes = read_bytes(cursor)?;
    let name = String::from_utf8(name_bytes).map_err(|e| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let dt_tag = read_u8(cursor)?;
    let data_type = decode_data_type(dt_tag)?;
    let nullable = read_u8(cursor)? != 0;
    let primary_key = read_u8(cursor)? != 0;
    let unique = read_u8(cursor)? != 0;

    let mut col = ColumnDef::new(name, data_type);
    col.set_flags_raw(nullable, primary_key, unique);

    if read_u8(cursor)? == 1 {
        let val_bytes = read_bytes(cursor)?;
        let mut val_cursor = Cursor::new(&val_bytes[..]);
        let val = decode_value(&mut val_cursor)?;
        col.set_default(Some(val));
    }

    if read_u8(cursor)? == 1 {
        let column_bytes = read_bytes(cursor)?;
        let column = String::from_utf8(column_bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        let op_tag = read_u8(cursor)?;
        let op = match op_tag {
            0 => ComparisonOp::Eq,
            1 => ComparisonOp::NotEq,
            2 => ComparisonOp::Lt,
            3 => ComparisonOp::Lte,
            4 => ComparisonOp::Gt,
            5 => ComparisonOp::Gte,
            _ => {
                return Err(DbError::corruption(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid comparison op tag",
                )));
            }
        };
        let val_bytes = read_bytes(cursor)?;
        let mut val_cursor = Cursor::new(&val_bytes[..]);
        let value = decode_value(&mut val_cursor)?;
        col.set_check(Some(crate::core::schema::column::CheckConstraint {
            column,
            op,
            value,
        }));
    }

    if read_u8(cursor)? == 1 {
        let table_bytes = read_bytes(cursor)?;
        let table = String::from_utf8(table_bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        let column_bytes = read_bytes(cursor)?;
        let column = String::from_utf8(column_bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        col.set_foreign_key(Some(crate::core::schema::column::ForeignKey {
            table,
            column,
        }));
    }

    Ok(col)
}

pub fn encode_table_schema(schema: &TableSchema) -> Vec<u8> {
    let mut buf = Vec::new();
    write_bytes(&mut buf, schema.name().as_bytes());
    write_u32(&mut buf, schema.columns().len() as u32);
    for col in schema.columns() {
        let col_bytes = encode_column_def(col);
        write_bytes(&mut buf, &col_bytes);
    }
    buf
}

pub fn decode_table_schema(cursor: &mut Cursor<&[u8]>) -> Result<TableSchema, DbError> {
    let name_bytes = read_bytes(cursor)?;
    let name = String::from_utf8(name_bytes).map_err(|e| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let col_count = read_u32(cursor)? as usize;
    if col_count > 1024 {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "too many columns",
        )));
    }
    let mut columns = Vec::with_capacity(col_count);
    for _ in 0..col_count {
        let col_bytes = read_bytes(cursor)?;
        let mut col_cursor = Cursor::new(&col_bytes[..]);
        let col = decode_column_def(&mut col_cursor)?;
        columns.push(col);
    }
    TableSchema::try_new(name, columns)
}

pub fn encode_row(row: &Row) -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, row.len() as u32);
    for value in row.values() {
        let value_bytes = encode_value(value);
        write_bytes(&mut buf, &value_bytes);
    }
    buf
}

pub fn decode_row(cursor: &mut Cursor<&[u8]>) -> Result<Row, DbError> {
    let value_count = read_u32(cursor)? as usize;
    if value_count > 1024 {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "too many values in row",
        )));
    }
    let mut values = Vec::with_capacity(value_count);
    for _ in 0..value_count {
        let value_bytes = read_bytes(cursor)?;
        let mut val_cursor = Cursor::new(&value_bytes[..]);
        let value = decode_value(&mut val_cursor)?;
        values.push(value);
    }
    Ok(Row::new(values))
}

pub fn encode_table(table: &Table) -> Vec<u8> {
    let mut buf = Vec::new();
    let schema_bytes = encode_table_schema(table.schema());
    write_bytes(&mut buf, &schema_bytes);

    write_u32(&mut buf, table.len() as u32);
    for row in table.rows() {
        let row_bytes = encode_row(row);
        write_bytes(&mut buf, &row_bytes);
    }
    buf
}

pub fn decode_table(cursor: &mut Cursor<&[u8]>) -> Result<Table, DbError> {
    let schema_bytes = read_bytes(cursor)?;
    let mut schema_cursor = Cursor::new(&schema_bytes[..]);
    let schema = decode_table_schema(&mut schema_cursor)?;

    let row_count = read_u32(cursor)? as usize;
    if row_count > 10_000_000 {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "too many rows",
        )));
    }
    let mut table = Table::new(schema);
    for _ in 0..row_count {
        let row_bytes = read_bytes(cursor)?;
        let mut row_cursor = Cursor::new(&row_bytes[..]);
        let row = decode_row(&mut row_cursor)?;
        table.insert(row)?;
    }
    Ok(table)
}

pub fn encode_catalog(catalog: &Catalog) -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32(&mut buf, FORMAT_VERSION);
    write_u32(&mut buf, catalog.len() as u32);
    for (name, table) in catalog.tables() {
        write_bytes(&mut buf, name.as_bytes());
        let table_bytes = encode_table(table);
        write_bytes(&mut buf, &table_bytes);
    }
    buf
}

pub fn decode_catalog(data: &[u8]) -> Result<Catalog, DbError> {
    let mut cursor = Cursor::new(data);
    let version = read_u32(&mut cursor)?;
    if version != FORMAT_VERSION {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unsupported format version {version}"),
        )));
    }
    decode_catalog_inner(&mut cursor)
}

fn decode_catalog_inner(cursor: &mut Cursor<&[u8]>) -> Result<Catalog, DbError> {
    let table_count = read_u32(cursor)? as usize;
    if table_count > 1024 {
        return Err(DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "too many tables",
        )));
    }
    let mut catalog = Catalog::new();
    for _ in 0..table_count {
        let name_bytes = read_bytes(cursor)?;
        let name = String::from_utf8(name_bytes).map_err(|e| {
            DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        let table_bytes = read_bytes(cursor)?;
        let table = decode_table(&mut Cursor::new(&table_bytes[..]))?;
        catalog.create_table(table.schema().clone())?;
        *catalog
            .get_table_mut(&name)
            .ok_or_else(|| DbError::table_not_found(&name))? = table;
    }
    Ok(catalog)
}
