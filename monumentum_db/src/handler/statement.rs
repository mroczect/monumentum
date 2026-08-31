use crate::core::schema::table_schema::TableSchema;
use crate::core::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub column: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    CreateTable(TableSchema),
    DropTable(String),
    Insert {
        table: String,
        values: Vec<Value>,
    },
    Select {
        table: String,
        columns: Vec<String>,
        where_clause: Option<WhereClause>,
    },
    Update {
        table: String,
        assignments: Vec<(String, Value)>,
        where_clause: Option<WhereClause>,
    },
    Delete {
        table: String,
        where_clause: Option<WhereClause>,
    },
}
