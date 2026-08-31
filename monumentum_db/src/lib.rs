pub mod error;
pub mod types;
pub use error::*;
pub use types::*;
#[must_use]
pub const fn add(left: u64, right: u64) -> u64 {
    left.wrapping_add(right)
}
