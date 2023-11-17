//! ICS 04: Channel implementation that facilitates communication between
//! applications and the chains those applications are built upon.

extern crate alloc;

pub mod channel;
pub mod error;
pub mod events;

pub mod msgs;
pub mod packet;
pub mod timeout;

pub mod acknowledgement;
pub mod commitment;
mod version;
pub use version::Version;

/// Re-exports ICS-04 channel proto types from the `ibc-proto-rs` crate
pub mod proto {
    pub use ibc_proto::google::protobuf::Any;
    pub use ibc_proto::ibc::core::channel::*;
    pub use ibc_proto::Protobuf;
}
