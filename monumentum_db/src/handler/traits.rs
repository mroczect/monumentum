use crate::error::DbError;
use crate::handler::result::QueryResult;
use crate::handler::statement::Statement;

pub trait Executor {
    fn execute(&mut self, statement: Statement) -> Result<QueryResult, DbError>;
}
