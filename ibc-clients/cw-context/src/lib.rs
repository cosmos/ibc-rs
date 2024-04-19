//! Contains types and implementations that are needed to integrate a light
//! client, built using ibc-rs, into CosmWasm contract. This crate functions as
//! a library, allowing users to import the ready-made `Context` object that is
//! generic across light clients, introduce their concrete client type and
//! integrate their assembled context into the CosmWasm contract's entrypoint.
//! NOTE: To utilize the CosmWasm contract developed using this library, hosting
//! environments must support CosmWasm module and be using one the versions of
//! `ibc-go` that supports the `08-wasm` proxy light client.

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

pub mod api;
pub mod context;
pub mod handlers;
pub mod types;
pub mod utils;
