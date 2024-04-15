//! Contains types and implementations that are needed to integrate a light
//!	client, built using ibc-rs, into CosmWasm contract. This crate functions as
//! a library, allowing users to import a ready-made `Context` object that is
//!	generic across light clients, introduce their concrete client type and
//!	integrate their custom context into the CosmWasm contract's entrypoint.
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

extern crate alloc;

pub mod api;
pub mod context;
pub mod handlers;
pub mod types;
pub mod utils;
