//! Implementation of Interchain Accounts (ICS27) application logic.
//!
//! Note: to be consistent with our naming convention defined in the
//! [`Core`](crate::core) module, we use the following terminology:
//! + We call "chain A" the chain that runs as the controller chain for the
//!   interchain account application
//! + We call "chain B" the chain that runs as the host chain for the interchain
//!   account application
//! In variable names:
//! + `_a` implies "belongs to chain A"
//! + `on_a` implies "stored on chain A"

pub mod account;
pub mod context;
pub mod controller;
pub mod error;
pub mod events;
pub mod host;
pub mod metadata;
pub mod packet;
pub mod port;

extern crate alloc;

/// Module identifier for the ICS27 application.
pub const MODULE_ID_STR: &str = "interchainaccounts";

/// ICS27 application current version.
pub const VERSION: &str = "ics27-1";

/// The successful string used for creating an acknowledgement status,
/// equivalent to `base64::encode(0x01)`.
pub const ACK_SUCCESS: &str = "AQ=="; //TODO: what's the result string?

use ibc_core::channel::types::acknowledgement::StatusValue;

/// Returns a successful acknowledgement status for the interchain accounts application.
pub fn ack_success() -> StatusValue {
    StatusValue::new(ACK_SUCCESS).expect("ack status value is never supposed to be empty")
}
