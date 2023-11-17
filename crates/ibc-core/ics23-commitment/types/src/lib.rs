//! ICS 23: Commitment implementation of a cryptographic scheme that verifies
//! state transitions between chains.

pub mod commitment;
pub mod error;
pub mod merkle;
#[cfg(feature = "serde")]
pub mod serializer;
pub mod specs;

/// Re-exports commitment proto types from the `ibc-proto-rs` crate
pub mod proto {
    pub use ibc_proto::ibc::core::commitment::*;
    pub use ibc_proto::{ics23, Protobuf};
}
