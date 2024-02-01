//! Implementation of the IBC [Non-Fungible Token
//! Transfer](https://github.com/cosmos/ibc/blob/main/spec/app/ics-721-nft-transfer/README.md)
//! (ICS-721) data structures.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

#[cfg(any(test, feature = "std"))]
extern crate std;

mod class;
mod data;
mod memo;
mod token;

pub mod events;
pub mod msgs;
pub use class::*;
pub use data::*;
pub mod packet;
pub use memo::*;
pub use token::*;
pub mod error;

/// Re-exports ICS-721 NFT transfer proto types from the `ibc-proto` crate.
pub mod proto {
    pub use ibc_proto::ibc::apps::nft_transfer;
}

/// Module identifier for the ICS-721 application.
pub const MODULE_ID_STR: &str = "nft_transfer";

/// The port identifier that the ICS-721 applications typically bind with.
pub const PORT_ID_STR: &str = "nft-transfer";

/// ICS-721 application current version.
pub const VERSION: &str = "ics721-1";

/// The successful string used for creating an acknowledgement status,
/// equivalent to `base64::encode(0x01)`.
pub const ACK_SUCCESS_B64: &str = "AQ==";

use ibc_core::channel::types::acknowledgement::StatusValue;

/// Returns a successful acknowledgement status for the NFT transfer application.
pub fn ack_success_b64() -> StatusValue {
    StatusValue::new(ACK_SUCCESS_B64).expect("ack status value is never supposed to be empty")
}
