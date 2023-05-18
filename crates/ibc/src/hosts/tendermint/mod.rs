//! Provides convenience implementations for Tendermint-based hosts

pub mod upgrade_proposal;

mod validate_self_client;
pub use validate_self_client::ValidateSelfClientContext;

/// ABCI store/query path for the IBC sub-store
pub const IBC_STORE_KEY: &str = "store/ibc/key";

/// ABCI store/query path for the upgrade sub-store
pub const UPGRADE_STORE_KEY: &str = "store/upgrade/key";
