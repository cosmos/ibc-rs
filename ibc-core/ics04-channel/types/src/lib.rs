//! Implementation of the Channel and Packet Semantics (ICS-04) data structures.
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

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod channel;
pub mod error;
pub mod events;

pub mod msgs;
pub mod packet;
pub mod timeout;

pub mod acknowledgement;
pub mod commitment;
mod version;
pub use version::Version;

/// Re-exports ICS-04 proto types from the `ibc-proto` crate
pub mod proto {
    pub use ibc_proto::ibc::core::channel::*;
}
