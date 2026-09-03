use crate::WorkbookError;
use monumentum_db::core::row::Row;
use monumentum_db::core::value::Value;
use monumentum_db::error::DbError;

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self, WorkbookError>;
}

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, WorkbookError>;
}

impl FromValue for Value {
    fn from_value(value: &Value) -> Result<Self, WorkbookError> {
        Ok(value.clone())
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self, WorkbookError> {
        match value {
            Value::Text(t) => Ok(t.as_str().to_string()),
            Value::Integer(i) => Ok(i.as_i64().to_string()),
            Value::Float(f) => Ok(f.as_f64().to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Null | Value::Blob(_) | Value::Formula(_) | _ => Err(WorkbookError::Db(
                DbError::type_mismatch("cannot convert value to String"),
            )),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Result<Self, WorkbookError> {
        match value {
            Value::Integer(i) => Ok(i.as_i64()),
            #[allow(clippy::cast_possible_truncation)]
            Value::Float(f) => Ok(f.as_f64() as Self),
            Value::Null
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => Err(WorkbookError::Db(DbError::type_mismatch(
                "cannot convert value to i64",
            ))),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Result<Self, WorkbookError> {
        match value {
            #[allow(clippy::cast_precision_loss)]
            Value::Integer(i) => Ok(i.as_i64() as Self),
            Value::Float(f) => Ok(f.as_f64()),
            Value::Null
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Boolean(_)
            | Value::Formula(_)
            | _ => Err(WorkbookError::Db(DbError::type_mismatch(
                "cannot convert value to f64",
            ))),
        }
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self, WorkbookError> {
        match value {
            Value::Boolean(b) => Ok(*b),
            Value::Null
            | Value::Integer(_)
            | Value::Float(_)
            | Value::Text(_)
            | Value::Blob(_)
            | Value::Formula(_)
            | _ => Err(WorkbookError::Db(DbError::type_mismatch(
                "cannot convert value to bool",
            ))),
        }
    }
}

impl FromRow for Value {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        row.get(0).cloned().ok_or(WorkbookError::InvalidReference)
    }
}

impl FromRow for String {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        let value = row.get(0).ok_or(WorkbookError::InvalidReference)?;
        Self::from_value(value)
    }
}

impl FromRow for i64 {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        let value = row.get(0).ok_or(WorkbookError::InvalidReference)?;
        Self::from_value(value)
    }
}

impl FromRow for f64 {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        let value = row.get(0).ok_or(WorkbookError::InvalidReference)?;
        Self::from_value(value)
    }
}

impl FromRow for bool {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        let value = row.get(0).ok_or(WorkbookError::InvalidReference)?;
        Self::from_value(value)
    }
}

impl FromRow for Vec<Value> {
    fn from_row(row: &Row) -> Result<Self, WorkbookError> {
        Ok(row.values().to_vec())
    }
}

macro_rules! impl_from_row_for_tuple {
    ($( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T),+> FromRow for ($($T,)+)
        where
            $($T: FromValue),+
        {
            fn from_row(row: &Row) -> Result<Self, WorkbookError> {
                Ok(($(
                    {
                        let value = row.get($idx).ok_or(WorkbookError::InvalidReference)?;
                        $T::from_value(value)?
                    },
                )+))
            }
        }
    };
}

impl_from_row_for_tuple!((0) -> T1;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4; (4) -> T5;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4; (4) -> T5; (5) -> T6;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4; (4) -> T5; (5) -> T6; (6) -> T7;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4; (4) -> T5; (5) -> T6; (6) -> T7; (7) -> T8;);
impl_from_row_for_tuple!((0) -> T1; (1) -> T2; (2) -> T3; (3) -> T4; (4) -> T5; (5) -> T6; (6) -> T7; (7) -> T8; (8) -> T9;);
