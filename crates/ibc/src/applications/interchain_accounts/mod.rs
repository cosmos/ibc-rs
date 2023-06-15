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

/// Module identifier for the ICS27 application.
pub const MODULE_ID_STR: &str = "interchainaccounts";

/// ICS27 application current version.
pub const VERSION: &str = "ics27-1";
