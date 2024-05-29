//! The CosmWasm contract implementation of the ICS-07 Tendermint light client
//! built using `ibc-rs`.
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

pub mod client_type;
pub mod entrypoint;
