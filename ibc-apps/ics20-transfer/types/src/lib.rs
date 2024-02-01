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

mod amount;
mod coin;
mod denom;
mod memo;

pub use amount::*;
pub use coin::*;
pub use denom::*;
pub mod error;
pub mod events;
pub mod msgs;
pub mod packet;
pub use memo::*;
/// Re-exports `U256` from `primitive-types` crate for convenience.
pub use primitive_types::U256;

/// Re-exports ICS-20 token transfer proto types from the `ibc-proto` crate.
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
