use super::Encode;
use crate::core::catalog::Catalog;
use crate::core::row::Row;
use crate::core::schema::column::{ColumnDef, ComparisonOp, DataType};
use crate::core::schema::table_schema::TableSchema;
use crate::core::table::Table;
use crate::error::DbError;

impl Encode for DataType {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        let tag: u8 = match self {
            Self::Null => 0,
            Self::Integer => 1,
            Self::Float => 2,
            Self::Text => 3,
            Self::Blob => 4,
            Self::Boolean => 5,
        };
        tag.encode(buf)
    }
}

impl Encode for ComparisonOp {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        let tag: u8 = match self {
            Self::Eq => 0,
            Self::NotEq => 1,
            Self::Lt => 2,
            Self::Lte => 3,
            Self::Gt => 4,
            Self::Gte => 5,
        };
        tag.encode(buf)
    }
}

impl Encode for ColumnDef {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.name().as_bytes().encode(buf)?;
        self.data_type().encode(buf)?;
        self.is_nullable().encode(buf)?;
        self.is_primary_key().encode(buf)?;
        self.is_unique().encode(buf)?;

        self.default_value().encode(buf)?;
        self.check_constraint().encode(buf)?;
        self.foreign_key().encode(buf)?;
        self.allowed_values().encode(buf)?;

        Ok(())
    }
}

impl Encode for crate::core::schema::column::CheckConstraint {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.column.as_bytes().encode(buf)?;
        self.op.encode(buf)?;
        self.value.encode(buf)
    }
}

impl Encode for crate::core::schema::column::ForeignKey {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.table.as_bytes().encode(buf)?;
        self.column.as_bytes().encode(buf)
    }
}

impl Encode for TableSchema {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.name().as_bytes().encode(buf)?;
        (self.columns().len() as u32).encode(buf)?;
        for col in self.columns() {
            col.encode(buf)?;
        }
        Ok(())
    }
}

impl Encode for Row {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        (self.len() as u32).encode(buf)?;
        for value in self.values() {
            value.encode(buf)?;
        }
        Ok(())
    }
}

impl Encode for Table {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.schema().encode(buf)?;
        self.is_read_only().encode(buf)?;
        (self.len() as u32).encode(buf)?;
        for row in self.rows() {
            row.encode(buf)?;
        }
        Ok(())
    }
}

impl Encode for Catalog {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        super::FORMAT_VERSION.encode(buf)?;
        (self.len() as u32).encode(buf)?;
        for (name, table) in self.tables() {
            name.as_bytes().encode(buf)?;
            table.encode(buf)?;
        }
        Ok(())
    }
}
