//! This module implements the processing logic for ICS3 (connection open
//! handshake) messages.
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

#[cfg(feature = "std")]
extern crate std;

pub mod delay;
pub mod handler;

/// Re-exports ICS-03 connection data structures from the `ibc-core-connection-types` crate
pub mod types {
    #[doc(inline)]
    pub use ibc_core_connection_types::*;
}
