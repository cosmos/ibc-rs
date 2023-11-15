//! Implementation of the [fungible token transfer module](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md) (ICS-20)
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod amount;
mod coin;
mod denom;
pub mod error;
pub mod events;
mod memo;
pub mod msgs;
pub mod packet;

pub use amount::*;
pub use coin::*;
pub use denom::*;
pub use memo::*;

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

use ibc::core::ics04_channel::acknowledgement::StatusValue;

/// Returns a successful acknowledgement status for the token transfer application.
pub fn ack_success_b64() -> StatusValue {
    StatusValue::new(ACK_SUCCESS_B64).expect("ack status value is never supposed to be empty")
}
