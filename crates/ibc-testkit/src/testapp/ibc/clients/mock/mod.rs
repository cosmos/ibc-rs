//! Definitions of ibc mock types used in testing.
pub mod client_state;
pub mod consensus_state;
pub mod header;
pub mod misbehaviour;

/// Re-exports mock proto types from the `ibc-proto` crate
pub mod proto {
    pub use ibc_proto::ibc::mock::*;
}
