use monumentum_handler::core::row::Row;
use monumentum_handler::core::value::Value;
use monumentum_handler::error::DbError;

mod cume_dist;
mod dense_rank;
mod first_value;
mod lag;
mod last_value;
mod lead;
mod nth_value;
mod ntile;
mod percent_rank;
mod rank;
mod row_number;

pub use self::cume_dist::CumeDistFunction;
pub use self::dense_rank::DenseRankFunction;
pub use self::first_value::FirstValueFunction;
pub use self::lag::LagFunction;
pub use self::last_value::LastValueFunction;
pub use self::lead::LeadFunction;
pub use self::nth_value::NthValueFunction;
pub use self::ntile::NtileFunction;
pub use self::percent_rank::PercentRankFunction;
pub use self::rank::RankFunction;
pub use self::row_number::RowNumberFunction;

pub trait WindowFunction: Send + Sync {
    fn name(&self) -> &'static str;
    fn evaluate(
        &self,
        partition: &[Row],
        current_idx: usize,
        args: &[Value],
        order_values: &[Option<Value>],
    ) -> Result<Value, DbError>;
}
