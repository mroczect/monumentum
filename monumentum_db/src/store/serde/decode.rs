use super::*;
use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::{ColumnDef, ComparisonOp};
use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use std::io::Cursor;

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

    if read_u8(cursor)? == 1 {
        let count = read_u32(cursor)? as usize;
        let mut allowed = Vec::with_capacity(count);
        for _ in 0..count {
            let val_bytes = read_bytes(cursor)?;
            let mut val_cursor = Cursor::new(&val_bytes[..]);
            let value = decode_value(&mut val_cursor)?;
            allowed.push(value);
        }
        col.set_allowed_values(Some(allowed));
    }

    Ok(col)
}

pub fn decode_table_schema(cursor: &mut Cursor<&[u8]>) -> Result<TableSchema, DbError> {
    let name_bytes = read_bytes(cursor)?;
    let name = String::from_utf8(name_bytes).map_err(|e| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let col_count_u32 = read_u32(cursor)?;
    let col_count = usize::try_from(col_count_u32).map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "column count too large for platform",
        ))
    })?;
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

pub fn decode_row(cursor: &mut Cursor<&[u8]>) -> Result<Row, DbError> {
    let value_count_u32 = read_u32(cursor)?;
    let value_count = usize::try_from(value_count_u32).map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "value count too large for platform",
        ))
    })?;
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

pub fn decode_table(cursor: &mut Cursor<&[u8]>) -> Result<Table, DbError> {
    let schema_bytes = read_bytes(cursor)?;
    let mut schema_cursor = Cursor::new(&schema_bytes[..]);
    let schema = decode_table_schema(&mut schema_cursor)?;

    let row_count_u32 = read_u32(cursor)?;
    let row_count = usize::try_from(row_count_u32).map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "row count too large for platform",
        ))
    })?;
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
    let table_count_u32 = read_u32(cursor)?;
    let table_count = usize::try_from(table_count_u32).map_err(|_| {
        DbError::corruption(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "table count too large for platform",
        ))
    })?;
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
