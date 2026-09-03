use super::{Decode, FORMAT_VERSION};
use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::{ColumnDef, ComparisonOp, DataType};
use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use crate::error::DbError;
use std::io::Cursor;

impl Decode for DataType {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let tag = u8::decode(cursor)?;
        match tag {
            0 => Ok(Self::Null),
            1 => Ok(Self::Integer),
            2 => Ok(Self::Float),
            3 => Ok(Self::Text),
            4 => Ok(Self::Blob),
            _ => Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid data type tag",
            ))),
        }
    }
}

impl Decode for ComparisonOp {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let tag = u8::decode(cursor)?;
        match tag {
            0 => Ok(Self::Eq),
            1 => Ok(Self::NotEq),
            2 => Ok(Self::Lt),
            3 => Ok(Self::Lte),
            4 => Ok(Self::Gt),
            5 => Ok(Self::Gte),
            _ => Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid comparison op tag",
            ))),
        }
    }
}

impl Decode for crate::core::schema::column::CheckConstraint {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let column = String::decode(cursor)?;
        let op = ComparisonOp::decode(cursor)?;
        let value = crate::core::value::Value::decode(cursor)?;
        Ok(Self { column, op, value })
    }
}

impl Decode for crate::core::schema::column::ForeignKey {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let table = String::decode(cursor)?;
        let column = String::decode(cursor)?;
        Ok(Self { table, column })
    }
}

impl Decode for ColumnDef {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let name = String::decode(cursor)?;
        let data_type = DataType::decode(cursor)?;
        let nullable = bool::decode(cursor)?;
        let primary_key = bool::decode(cursor)?;
        let unique = bool::decode(cursor)?;

        let mut col = ColumnDef::new(name, data_type);
        col.set_flags_raw(nullable, primary_key, unique);

        col.set_default(Option::<crate::core::value::Value>::decode(cursor)?);
        col.set_check(Option::<crate::core::schema::column::CheckConstraint>::decode(cursor)?);
        col.set_foreign_key(Option::<crate::core::schema::column::ForeignKey>::decode(
            cursor,
        )?);
        col.set_allowed_values(Option::<Vec<crate::core::value::Value>>::decode(cursor)?);

        Ok(col)
    }
}

impl Decode for TableSchema {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let name = String::decode(cursor)?;
        let col_count = u32::decode(cursor)? as usize;
        if col_count > 1024 {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many columns",
            )));
        }
        let mut columns = Vec::with_capacity(col_count);
        for _ in 0..col_count {
            columns.push(ColumnDef::decode(cursor)?);
        }
        TableSchema::try_new(name, columns)
    }
}

impl Decode for Row {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let value_count = u32::decode(cursor)? as usize;
        if value_count > 1024 {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many values in row",
            )));
        }
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(crate::core::value::Value::decode(cursor)?);
        }
        Ok(Row::new(values))
    }
}

impl Decode for Table {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let schema = TableSchema::decode(cursor)?;
        let read_only = bool::decode(cursor)?;
        let row_count = u32::decode(cursor)? as usize;
        if row_count > 10_000_000 {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many rows",
            )));
        }
        let mut table = Table::new(schema);
        table.set_read_only(read_only);
        for _ in 0..row_count {
            let row = Row::decode(cursor)?;
            table.insert(row).map_err(DbError::corruption)?;
        }
        Ok(table)
    }
}

impl Decode for Catalog {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, DbError> {
        let version = u32::decode(cursor)?;
        if version != FORMAT_VERSION {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported format version {version}"),
            )));
        }
        let table_count = u32::decode(cursor)? as usize;
        if table_count > 1024 {
            return Err(DbError::corruption(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many tables",
            )));
        }
        let mut catalog = Catalog::new();
        for _ in 0..table_count {
            let name = String::decode(cursor)?;
            let table = Table::decode(cursor)?;
            catalog.create_table(table.schema().clone())?;
            *catalog
                .get_table_mut(&name)
                .ok_or_else(|| DbError::table_not_found(&name))? = table;
        }
        Ok(catalog)
    }
}
