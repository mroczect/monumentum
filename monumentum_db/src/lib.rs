pub mod error;
pub use error::*;
#[must_use]
pub const fn add(left: u64, right: u64) -> u64 {
    left.wrapping_add(right)
}
