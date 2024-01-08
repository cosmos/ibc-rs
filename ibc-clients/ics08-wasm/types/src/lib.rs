//! Definitions of domain types used in the ICS-08 Wasm light client implementation.
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod msgs;

#[cfg(feature = "cosmwasm")]
pub mod serializer;

use ibc_primitives::prelude::Vec;
pub type Bytes = Vec<u8>;

pub static SUBJECT_PREFIX: &[u8] = "subject/".as_bytes();
pub static SUBSTITUTE_PREFIX: &[u8] = "substitute/".as_bytes();
