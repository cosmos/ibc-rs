//! Implementation of mocks for context, host chain, and client.

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod client_state;
#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod consensus_state;
#[cfg(any(test, feature = "mocks"))]
pub mod context;
#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod header;
#[cfg(any(test, feature = "mocks"))]
pub mod host;
#[cfg(any(test, feature = "mocks"))]
pub mod ics18_relayer;
#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod misbehaviour;
