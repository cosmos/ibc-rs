//! Provides convenience traits and implementations for Tendermint-based hosts
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
pub mod upgrade_proposal;

pub mod utils;

mod validate_self_client;
pub use validate_self_client::ValidateSelfClientContext;

/// Re-exports necessary proto types for implementing the tendermint client
/// upgradeability feature.
pub mod proto {
    pub use ibc_proto::cosmos::upgrade::*;
}

/// ABCI store/query path for the IBC sub-store
pub const IBC_QUERY_PATH: &str = "store/ibc/key";

/// ABCI store/query path for the upgrade sub-store
pub const SDK_UPGRADE_QUERY_PATH: &str = "store/upgrade/key";
