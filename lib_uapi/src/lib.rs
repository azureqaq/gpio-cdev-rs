#[cfg(not(any(feature = "v1", feature = "v2")))]
std::compile_error!("At least one feature v1 or v2 must be enabled.");

mod common;
pub mod error;
mod macros;
#[cfg(feature = "v1")]
mod v1;
#[cfg(feature = "v2")]
mod v2;

pub use common::*;
