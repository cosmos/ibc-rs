//! Implementation of the Connection Semantics (ICS-03) data structures.
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

mod connection;
pub use connection::*;

pub mod error;
pub mod events;
pub mod msgs;
pub mod version;

/// Re-exports ICS-03 proto types from the `ibc-proto` crate for added
/// convenience
pub mod proto {
    pub use ibc_proto::ibc::core::connection::*;
}
