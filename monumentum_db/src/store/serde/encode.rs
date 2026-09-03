use super::{FORMAT_VERSION, encode_data_type, encode_value, write_bytes, write_u8, write_u32};
use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::{ColumnDef, ComparisonOp};
use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use crate::error::DbError;

pub fn encode_column_def(col: &ColumnDef) -> Result<Vec<u8>, DbError> {
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

    match col.allowed_values() {
        Some(values) => {
            write_u8(&mut buf, 1);
            let count = u32::try_from(values.len())
                .map_err(|_| DbError::invalid_operation("too many allowed values"))?;
            write_u32(&mut buf, count);
            for v in values {
                let val_bytes = encode_value(v);
                write_bytes(&mut buf, &val_bytes);
            }
        }
        None => write_u8(&mut buf, 0),
    }

    Ok(buf)
}

pub fn encode_table_schema(schema: &TableSchema) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::new();
    write_bytes(&mut buf, schema.name().as_bytes());
    let col_count = u32::try_from(schema.columns().len())
        .map_err(|_| DbError::invalid_operation("too many columns"))?;
    write_u32(&mut buf, col_count);
    for col in schema.columns() {
        let col_bytes = encode_column_def(col)?;
        write_bytes(&mut buf, &col_bytes);
    }
    Ok(buf)
}

pub fn encode_row(row: &Row) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::new();
    let value_count =
        u32::try_from(row.len()).map_err(|_| DbError::invalid_operation("row too large"))?;
    write_u32(&mut buf, value_count);
    for value in row.values() {
        let value_bytes = encode_value(value);
        write_bytes(&mut buf, &value_bytes);
    }
    Ok(buf)
}

pub fn encode_table(table: &Table) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::new();
    let schema_bytes = encode_table_schema(table.schema())?;
    write_bytes(&mut buf, &schema_bytes);

    write_u8(&mut buf, table.is_read_only() as u8);

    let row_count =
        u32::try_from(table.len()).map_err(|_| DbError::invalid_operation("too many rows"))?;
    write_u32(&mut buf, row_count);
    for row in table.rows() {
        let row_bytes = encode_row(row)?;
        write_bytes(&mut buf, &row_bytes);
    }
    Ok(buf)
}

pub fn encode_catalog(catalog: &Catalog) -> Result<Vec<u8>, DbError> {
    let mut buf = Vec::new();
    write_u32(&mut buf, FORMAT_VERSION);
    let table_count =
        u32::try_from(catalog.len()).map_err(|_| DbError::invalid_operation("too many tables"))?;
    write_u32(&mut buf, table_count);
    for (name, table) in catalog.tables() {
        write_bytes(&mut buf, name.as_bytes());
        let table_bytes = encode_table(table)?;
        write_bytes(&mut buf, &table_bytes);
    }
    Ok(buf)
}
