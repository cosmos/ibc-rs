#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![no_std]
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

extern crate std;

pub mod hosts;
pub mod relayer;
pub mod testapp;
pub mod utils;

// Re-export the `mock` types from `ibc` crate.
pub use ibc::mock;
