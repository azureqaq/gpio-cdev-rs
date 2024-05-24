#[cfg(all(feature = "v1", feature = "v2"))]
compile_error!("Features `v1` and `v2` cannot be enabled at the same time.");

#[cfg(not(any(feature = "v1", feature = "v2")))]
compile_error!("One of the features `v1` or `v2` must be enabled.");

pub mod chip;
mod error;
mod ffi;
pub mod line;
mod macros;

pub use error::{Error, IoctlKind, Result};
