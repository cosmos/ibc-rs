//! Helper Context for handling upgrade client proposals. A sample
//! implementation can be found in the
//! [Basecoin-rs](https://github.com/informalsystems/basecoin-rs) repository.

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use crate::core::ics24_host::path::UpgradeClientPath;

use super::error::UpgradeError;
use super::proposal::Plan;

/// Helper context for validating upgrades, providing methods to retrieve
/// upgrade plans and upgraded client and consensus states.
pub trait UpgradeValidationContext {
    /// Returns the upgrade plan that is scheduled and not have been executed yet.
    fn upgrade_plan(&self) -> Result<Plan, UpgradeError>;

    /// Returns the upgraded client state at the specified upgrade path.
    fn upgraded_client_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<TmClientState, UpgradeError>;

    /// Returns the upgraded consensus state at the specified upgrade path.
    fn upgraded_consensus_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<TmConsensusState, UpgradeError>;
}

/// Helper context for executing upgrades, providing methods to schedule
/// upgrades and store upgraded client and consensus states.
pub trait UpgradeExecutionContext: UpgradeValidationContext {
    /// Schedules an upgrade based on the specified plan. If there is another `Plan` it should be overwritten.
    fn schedule_upgrade(&mut self, plan: Plan) -> Result<(), UpgradeError>;

    /// Clears the upgrade plan at the specified height.
    fn clear_upgrade_plan(&mut self, plan_height: u64) -> Result<(), UpgradeError>;

    /// Stores the upgraded client state at the specified upgrade path.
    fn store_upgraded_client_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        client_state: TmClientState,
    ) -> Result<(), UpgradeError>;

    /// Stores the upgraded consensus state at the specified upgrade path.
    fn store_upgraded_consensus_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        consensus_state: TmConsensusState,
    ) -> Result<(), UpgradeError>;
}
