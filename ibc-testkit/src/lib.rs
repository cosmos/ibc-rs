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

#[cfg(feature = "std")]
extern crate std;

pub mod fixtures;
pub mod hosts;
pub mod relayer;
pub mod testapp;
