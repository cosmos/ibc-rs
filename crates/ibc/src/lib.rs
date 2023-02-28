// TODO: disable unwraps:
// https://github.com/informalsystems/ibc-rs/issues/987
// #![cfg_attr(not(test), deny(clippy::unwrap_used))]
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
// https://github.com/cosmos/ibc-rs/issues/342
#![allow(clippy::result_large_err)]
//! This library implements the InterBlockchain Communication (IBC) protocol in Rust. IBC is
//! a distributed protocol that enables communication between distinct sovereign blockchains.
//! Loose analogies may be drawn between the IBC protocol and the TCP/UDP protocols that enable
//! communication over the internet via packet streaming. Indeed, IBC also encodes the notion of
//! ordered and unordered packet streams.
//!
//! The layout of this crate mirrors the classification of the [Interchain
//! Standards][ics-standards]. The classification consists of [Core][core], [Clients][clients],
//! and [Applications][applications].
//!
//! `Core` consists of the designs and logic pertaining to the transport, authentication, and
//! ordering layers of the IBC protocol, the fundamental pieces.
//!
//! `Clients` consists of implementations of client verification algorithms (following the base
//! client interface that is defined in `Core`) for specific types of chains. A chain uses these
//! verification algorithms to verify the state of remote chains.
//!
//! `Applications` consists of various packet encoding and processing semantics which underpin the
//! various types of transactions that users can perform on any IBC-compliant chain.
//!
//! [core]: https://github.com/cosmos/ibc-rs/tree/main/crates/ibc/src/core
//! [clients]: https://github.com/cosmos/ibc-rs/tree/main/crates/ibc/src/clients
//! [applications]: https://github.com/cosmos/ibc-rs/tree/main/crates/ibc/src/applications
//! [ics-standards]: https://github.com/cosmos/ibc#interchain-standards

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod prelude;

pub mod applications;
pub mod clients;
pub mod core;
pub mod dynamic_typing;
mod erased;
pub mod events;
pub mod hosts;
pub mod signer;
pub mod timestamp;
pub mod tx_msg;
pub mod utils;

#[cfg(feature = "serde")]
mod serializers;

/// Re-export of ICS 002 Height domain type
pub type Height = crate::core::ics02_client::height::Height;

#[cfg(test)]
mod test;

#[cfg(any(test, feature = "mocks"))]
pub mod test_utils;

pub mod mock; // Context mock, the underlying host chain, and client types: for testing all handlers.
