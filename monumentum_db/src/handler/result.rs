use crate::core::row::Row;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryResult {
    Empty,
    Rows(Vec<Row>),
    AffectedRows(usize),
}
