#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod context;
pub mod fixtures;
pub mod hosts;
pub mod relayer;
pub mod testapp;
pub mod utils;

// `ibc::apps::transfer::handler::send_transfer` requires `serde`
#[cfg(feature = "serde")]
pub mod integration;
