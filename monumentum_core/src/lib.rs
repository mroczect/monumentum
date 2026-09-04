#![forbid(unsafe_code)]

extern crate alloc;

use monumentum_handler as _;

pub mod buffer_pool;
pub mod catalog;
pub mod index;
pub mod page;
pub mod pager;
pub mod serde;
pub mod store;
pub mod table;
