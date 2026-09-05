use super::{Encode, FORMAT_VERSION};
use crate::catalog::Catalog;
use crate::table::Table;
use monumentum_handler::{
    core::schema::column::{CheckConstraint, ColumnDef, ComparisonOp, DataType, ForeignKey},
    core::schema::table_schema::TableSchema,
    error::DbError,
};

use monumentum_handler::core::row::Row;
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

impl Encode for CheckConstraint {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.column.as_bytes().encode(buf)?;
        self.op.encode(buf)?;
        self.value.encode(buf)
    }
}

impl Encode for ForeignKey {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.table.as_bytes().encode(buf)?;
        self.column.as_bytes().encode(buf)
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

impl Encode for TableSchema {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.name().as_bytes().encode(buf)?;
        let len = u32::try_from(self.columns().len())
            .map_err(|e| DbError::invalid_operation(format!("too many columns: {e}")))?;
        len.encode(buf)?;
        for col in self.columns() {
            col.encode(buf)?;
        }
        Ok(())
    }
}

impl Encode for Table {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        self.schema().encode(buf)?;
        self.is_read_only().encode(buf)?;
        self.data_page_id().encode(buf)?;
        self.index_root_page_id().encode(buf)?;
        self.next_row_id().encode(buf)
    }
}

impl Encode for Catalog {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        FORMAT_VERSION.encode(buf)?;
        let len = u32::try_from(self.len())
            .map_err(|e| DbError::invalid_operation(format!("catalog too large: {e}")))?;
        len.encode(buf)?;
        for (name, table) in self.tables() {
            name.as_bytes().encode(buf)?;
            table.encode(buf)?;
        }
        Ok(())
    }
}

impl Encode for Row {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<(), DbError> {
        let len = u32::try_from(self.len())
            .map_err(|e| DbError::invalid_operation(format!("row too large: {e}")))?;
        len.encode(buf)?;
        for value in self.values() {
            value.encode(buf)?;
        }
        Ok(())
    }
}
