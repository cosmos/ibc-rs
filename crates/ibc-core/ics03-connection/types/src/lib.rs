//! ICS 03: Connection implementation for connecting a client
//! on the local chain with a client on a remote chain.
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

#[cfg(feature = "std")]
extern crate std;

mod connection;
pub use connection::*;

pub mod error;
pub mod events;
pub mod msgs;
pub mod version;

pub mod primitives {
    #[doc(inline)]
    pub use ibc_primitives::*;
}

pub mod proto {
    pub use ibc_proto::google::protobuf::Any;
    pub use ibc_proto::ibc::core::connection::*;
    pub use ibc_proto::Protobuf;
}
