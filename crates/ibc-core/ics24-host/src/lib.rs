//! Provides essential top-level traits designed for the seamless integration of
//! host chains with ibc-rs. It streamlines access to the host's storage,
//! facilitating the efficient retrieval of states and metadata crucial for the
//! execution of IBC logics.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

#[cfg(any(test, feature = "std"))]
extern crate std;

pub(crate) mod utils;

mod context;
pub use context::*;

/// Re-exports ICS-24 data structures from `ibc-core-host-types` crate.
pub mod types {
    #[doc(inline)]
    pub use ibc_core_host_types::*;
}
