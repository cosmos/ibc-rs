//! Contains primitives types and traits common to various IBC components.
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

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod prelude;
pub mod utils;

mod traits;
pub use traits::*;

mod types;
pub use types::*;

// Helper module for serializing and deserializing types through the `String`
// primarily used by IBC applications.
#[cfg(feature = "serde")]
pub mod serializers;

pub mod proto {
    pub use ibc_proto::google::protobuf::{Any, Duration, Timestamp};
    pub use ibc_proto::Protobuf;
}
