//! Implementation of the IBC [fungible token transfer](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md) (ICS-20) data structures.
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

#[cfg(feature = "serde")]
mod amount;
#[cfg(feature = "serde")]
pub use amount::*;
#[cfg(feature = "serde")]
mod coin;
#[cfg(feature = "serde")]
pub use coin::*;
#[cfg(feature = "serde")]
mod denom;
#[cfg(feature = "serde")]
pub use denom::*;
#[cfg(feature = "serde")]
pub mod events;
#[cfg(feature = "serde")]
pub mod msgs;
#[cfg(feature = "serde")]
pub mod packet;

#[cfg(feature = "serde")]
pub(crate) mod serializers;

pub mod error;
mod memo;
pub use memo::*;

/// Re-exports ICS-20 token transfer proto types from the `ibc-proto-rs` crate
/// for added convenience
pub mod proto {
    pub use ibc_proto::ibc::apps::transfer;
}

/// Module identifier for the ICS20 application.
pub const MODULE_ID_STR: &str = "transfer";

/// The port identifier that the ICS20 applications
/// typically bind with.
pub const PORT_ID_STR: &str = "transfer";

/// ICS20 application current version.
pub const VERSION: &str = "ics20-1";

/// The successful string used for creating an acknowledgement status,
/// equivalent to `base64::encode(0x01)`.
pub const ACK_SUCCESS_B64: &str = "AQ==";

use ibc_core::channel::types::acknowledgement::StatusValue;

/// Returns a successful acknowledgement status for the token transfer application.
pub fn ack_success_b64() -> StatusValue {
    StatusValue::new(ACK_SUCCESS_B64).expect("ack status value is never supposed to be empty")
}
