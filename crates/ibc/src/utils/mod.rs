//! Various utilities used internally

#[cfg(any(test, feature = "test-utils"))]
pub mod dummy;
pub(crate) mod macros;
pub(crate) mod pretty;
