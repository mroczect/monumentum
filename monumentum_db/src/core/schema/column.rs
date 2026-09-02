use crate::core::value::Value;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Null,
    Integer,
    Float,
    Text,
    Blob,
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
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn set_nullable(&mut self, value: bool) {
        self.nullable = value;
        if value {
            self.primary_key = false;
        }
    }

    pub fn set_primary_key(&mut self, value: bool) {
        self.primary_key = value;
        if value {
            self.nullable = false;
            self.unique = true;
        }
    }

    pub fn set_unique(&mut self, value: bool) {
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
}
