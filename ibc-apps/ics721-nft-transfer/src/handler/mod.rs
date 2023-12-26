//! Implements IBC handlers responsible for processing Non-Fungible Token
//! Transfers (ICS-721) messages.
mod send_transfer;

pub use send_transfer::*;
