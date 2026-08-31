pub mod core;
pub mod error;
pub mod store;
pub mod types;

pub use core::*;
pub use error::*;
pub use store::*;
pub use types::*;
#[must_use]
pub const fn add(left: u64, right: u64) -> u64 {
    left.wrapping_add(right)
}
