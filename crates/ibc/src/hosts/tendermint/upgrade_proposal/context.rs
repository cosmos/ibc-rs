//! Helper Context for handling upgrade client proposals.
//!
//! Currently. this interface has been defined to support Tendermint-based
//! chains. You can check out a sample implementation in the
//! [Basecoin-rs](https://github.com/informalsystems/basecoin-rs) repository.
//! If it proves to be generic enough, we may move it to the ICS02 section.

use super::Plan;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::UpgradeClientError;
use crate::core::ics02_client::{ClientExecutionContext, ClientValidationContext};
use crate::core::ics24_host::path::UpgradeClientPath;

/// Helper context to validate client upgrades, providing methods to retrieve
/// an upgrade plan and related upgraded client and consensus states.
pub trait UpgradeValidationContext {
    type V: ClientValidationContext;
    type E: ClientExecutionContext;
    type AnyConsensusState: ConsensusState;
    type AnyClientState: ClientState<Self::V, Self::E>;

    /// Returns the upgrade plan that is scheduled and not have been executed yet.
    fn upgrade_plan(&self) -> Result<Plan, UpgradeClientError>;

    /// Returns the upgraded client state at the specified upgrade path.
    fn upgraded_client_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<Self::AnyClientState, UpgradeClientError>;

    /// Returns the upgraded consensus state at the specified upgrade path.
    fn upgraded_consensus_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<Self::AnyConsensusState, UpgradeClientError>;
}

/// Helper context to execute client upgrades, providing methods to schedule
/// an upgrade and store related upgraded client and consensus states.
pub trait UpgradeExecutionContext: UpgradeValidationContext {
    /// Schedules an upgrade based on the specified plan. If there is another `Plan` it should be overwritten.
    fn schedule_upgrade(&mut self, plan: Plan) -> Result<(), UpgradeClientError>;

    /// Clears the upgrade plan at the specified height.
    fn clear_upgrade_plan(&mut self, plan_height: u64) -> Result<(), UpgradeClientError>;

    /// Stores the upgraded client state at the specified upgrade path.
    fn store_upgraded_client_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        client_state: Self::AnyClientState,
    ) -> Result<(), UpgradeClientError>;

    /// Stores the upgraded consensus state at the specified upgrade path.
    fn store_upgraded_consensus_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), UpgradeClientError>;
}
