//! This crate functions as an intermediary layer between the storage of host
//! chains and an IBC client implementation, providing developers with necessary
//! traits to craft their custom light clients. It streamlines the process of
//! integrating light clients with the host, enabling interaction with the store
//! for pertinent client state transitions.
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

#[cfg(feature = "std")]
extern crate std;

pub mod client_state;
pub mod consensus_state;

mod context;
pub use context::*;

/// Trait preludes for the ICS-02 client implementation.
pub mod prelude {
    pub use crate::client_state::*;
    pub use crate::consensus_state::*;
    pub use crate::context::*;
}

pub mod types {
    #[doc(inline)]
    pub use ibc_core_client_types::*;
}
