//! Exposes IBC handler entry points for an integrated IBC core modules. These
//! entry points are responsible for processing incoming IBC messages,
//! performing validation, and execution logics by invoking the appropriate
//! module handler.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod entrypoint;

/// Re-export IBC handler types from `ibc-core-handler-types` crate.
pub mod types {
    #[doc(inline)]
    pub use ibc_core_handler_types::*;
}
