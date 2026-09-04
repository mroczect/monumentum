use crate::core::value::Value;
use crate::error::DbError;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Null,
    Integer,
    Float,
    Text,
    Blob,
    Boolean,
}

impl DataType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Null => "NULL",
            Self::Integer => "INTEGER",
            Self::Float => "FLOAT",
            Self::Text => "TEXT",
            Self::Blob => "BLOB",
            Self::Boolean => "BOOLEAN",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckConstraint {
    pub column: String,
    pub op: ComparisonOp,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    name: String,
    data_type: DataType,
    nullable: bool,
    primary_key: bool,
    unique: bool,
    default_value: Option<Value>,
    check_constraint: Option<CheckConstraint>,
    foreign_key: Option<ForeignKey>,
    allowed_values: Option<Vec<Value>>,
}

impl ColumnDef {
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            primary_key: false,
            unique: false,
            default_value: None,
            check_constraint: None,
            foreign_key: None,
            allowed_values: None,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn data_type(&self) -> &DataType {
        &self.data_type
    }

    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.nullable
    }

    #[must_use]
    pub const fn is_primary_key(&self) -> bool {
        self.primary_key
    }

    #[must_use]
    pub const fn is_unique(&self) -> bool {
        self.unique
    }

    #[must_use]
    pub const fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    #[must_use]
    pub const fn check_constraint(&self) -> Option<&CheckConstraint> {
        self.check_constraint.as_ref()
    }

    #[must_use]
    pub const fn foreign_key(&self) -> Option<&ForeignKey> {
        self.foreign_key.as_ref()
    }

    #[must_use]
    pub const fn allowed_values(&self) -> Option<&Vec<Value>> {
        self.allowed_values.as_ref()
    }

    pub const fn set_nullable(&mut self, value: bool) {
        self.nullable = value;
        if value {
            self.primary_key = false;
        }
    }

    pub const fn set_primary_key(&mut self, value: bool) {
        self.primary_key = value;
        if value {
            self.nullable = false;
            self.unique = true;
        }
    }

    pub const fn set_unique(&mut self, value: bool) {
        self.unique = value;
    }

    pub fn set_default(&mut self, value: Option<Value>) {
        self.default_value = value;
    }

    pub fn set_check(&mut self, constraint: Option<CheckConstraint>) {
        self.check_constraint = constraint;
    }

    pub fn set_foreign_key(&mut self, fk: Option<ForeignKey>) {
        self.foreign_key = fk;
    }

    pub fn set_allowed_values(&mut self, values: Option<Vec<Value>>) {
        self.allowed_values = values;
    }

    pub fn validate_value(&self, value: &Value) -> Result<(), DbError> {
        if value.is_null() {
            if !self.nullable {
                return Err(DbError::constraint_violation(
                    crate::error::ErrorKind::NotNullViolation,
                    format!("column '{}' is not nullable", self.name),
                    Some(self.name.clone()),
                    None,
                ));
            }
            return Ok(());
        }

        let type_ok = match self.data_type {
            DataType::Null => false,
            DataType::Integer => value.is_integer(),
            DataType::Float => value.is_float(),
            DataType::Text => value.is_text(),
            DataType::Blob => value.is_blob(),
            DataType::Boolean => value.is_boolean(),
        };

        if !type_ok {
            return Err(DbError::type_mismatch(format!(
                "column '{}' expects {}, got {}",
                self.name,
                self.data_type,
                value.type_name()
            )));
        }

        if let Some(check) = &self.check_constraint
            && !evaluate_check_value(value, check)
        {
            return Err(DbError::constraint_violation(
                crate::error::ErrorKind::CheckViolation,
                format!("check constraint failed for column '{}'", self.name),
                Some(self.name.clone()),
                None,
            ));
        }

        if let Some(allowed) = &self.allowed_values
            && !allowed.contains(value)
        {
            return Err(DbError::constraint_violation(
                crate::error::ErrorKind::CheckViolation,
                format!(
                    "value is not in the allowed list for column '{}'",
                    self.name
                ),
                Some(self.name.clone()),
                None,
            ));
        }

        Ok(())
    }
}

fn evaluate_check_value(value: &Value, check: &CheckConstraint) -> bool {
    match (&value, &check.value) {
        (Value::Integer(a), Value::Integer(b)) => match check.op {
            ComparisonOp::Eq => a.as_i64() == b.as_i64(),
            ComparisonOp::NotEq => a.as_i64() != b.as_i64(),
            ComparisonOp::Lt => a.as_i64() < b.as_i64(),
            ComparisonOp::Lte => a.as_i64() <= b.as_i64(),
            ComparisonOp::Gt => a.as_i64() > b.as_i64(),
            ComparisonOp::Gte => a.as_i64() >= b.as_i64(),
        },
        (Value::Float(a), Value::Float(b)) => match check.op {
            ComparisonOp::Eq => a.as_f64() == b.as_f64(),
            ComparisonOp::NotEq => a.as_f64() != b.as_f64(),
            ComparisonOp::Lt => a.as_f64() < b.as_f64(),
            ComparisonOp::Lte => a.as_f64() <= b.as_f64(),
            ComparisonOp::Gt => a.as_f64() > b.as_f64(),
            ComparisonOp::Gte => a.as_f64() >= b.as_f64(),
        },
        (Value::Text(a), Value::Text(b)) => match check.op {
            ComparisonOp::Eq => a.as_str() == b.as_str(),
            ComparisonOp::NotEq => a.as_str() != b.as_str(),
            ComparisonOp::Lt => a.as_str() < b.as_str(),
            ComparisonOp::Lte => a.as_str() <= b.as_str(),
            ComparisonOp::Gt => a.as_str() > b.as_str(),
            ComparisonOp::Gte => a.as_str() >= b.as_str(),
        },
        (Value::Blob(a), Value::Blob(b)) => match check.op {
            ComparisonOp::Eq => a.as_slice() == b.as_slice(),
            ComparisonOp::NotEq => a.as_slice() != b.as_slice(),
            ComparisonOp::Lt | ComparisonOp::Lte | ComparisonOp::Gt | ComparisonOp::Gte => false,
        },
        (Value::Null, _)
        | (Value::Integer(_), _)
        | (Value::Float(_), _)
        | (Value::Text(_), _)
        | (Value::Blob(_), _)
        | (Value::Boolean(_), _) => false,
    }
}

pub trait Column {
    fn name(&self) -> &str;
    fn data_type(&self) -> &DataType;
    fn is_nullable(&self) -> bool;
    fn is_primary_key(&self) -> bool;
    fn is_unique(&self) -> bool;
}

impl Column for ColumnDef {
    fn name(&self) -> &str {
        self.name()
    }
    fn data_type(&self) -> &DataType {
        self.data_type()
    }
    fn is_nullable(&self) -> bool {
        self.is_nullable()
    }
    fn is_primary_key(&self) -> bool {
        self.is_primary_key()
    }
    fn is_unique(&self) -> bool {
        self.is_unique()
    }
}

pub trait ColumnIndex<T: ?Sized> {
    fn index(&self, container: &T) -> Result<usize, DbError>;
}

impl ColumnIndex<crate::core::row::Row> for usize {
    fn index(&self, row: &crate::core::row::Row) -> Result<usize, DbError> {
        let len = row.len();
        if *self >= len {
            return Err(DbError::invalid_operation(format!(
                "column index {} out of bounds (len {})",
                self, len
            )));
        }
        Ok(*self)
    }
}

impl ColumnIndex<crate::core::schema::table_schema::TableSchema> for usize {
    fn index(
        &self,
        schema: &crate::core::schema::table_schema::TableSchema,
    ) -> Result<usize, DbError> {
        let len = schema.columns().len();
        if *self >= len {
            return Err(DbError::invalid_operation(format!(
                "column index {} out of bounds (len {})",
                self, len
            )));
        }
        Ok(*self)
    }
}

impl ColumnIndex<crate::core::schema::table_schema::TableSchema> for &str {
    fn index(
        &self,
        schema: &crate::core::schema::table_schema::TableSchema,
    ) -> Result<usize, DbError> {
        schema
            .column_index(self)
            .ok_or_else(|| DbError::column_not_found(*self))
    }
}

impl ColumnIndex<crate::core::table::Table> for &str {
    fn index(&self, table: &crate::core::table::Table) -> Result<usize, DbError> {
        table
            .schema()
            .column_index(self)
            .ok_or_else(|| DbError::column_not_found(*self))
    }
}
