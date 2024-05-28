//! FFI bindings for [gpio.h](https://elixir.bootlin.com/linux/v6.9.2/source/include/uapi/linux/gpio.h)

/// Common bindings that are version-agnostic.
pub(crate) mod common;
/// GPIO v1 bindings.
///
/// GPIO v1 is deprecated and should not be used.
#[cfg(feature = "v1")]
pub(crate) mod v1;

/// GPIO v2 bindings.
#[cfg(feature = "v2")]
pub(crate) mod v2;
