//! Helper Context for handling upgrade client proposals.
//!
//! Currently. this interface has been defined to support Tendermint-based
//! chains. You can check out a sample implementation in the
//! [Basecoin-rs](https://github.com/informalsystems/basecoin-rs) repository.
//! If it proves to be generic enough, we may move it to the ICS02 section.

use ibc_core_client_context::ClientValidationContext;
use ibc_core_client_types::error::UpgradeClientError;
use ibc_core_host_types::path::UpgradeClientPath;

use super::Plan;

pub type UpgradedClientStateRef<T> =
    <<T as UpgradeValidationContext>::V as ClientValidationContext>::ClientStateRef;

pub type UpgradedConsensusStateRef<T> =
    <<T as UpgradeValidationContext>::V as ClientValidationContext>::ConsensusStateRef;

/// Helper context to validate client upgrades, providing methods to retrieve
/// an upgrade plan and related upgraded client and consensus states.
pub trait UpgradeValidationContext {
    type V: ClientValidationContext;

    /// Returns the upgrade plan that is scheduled and not have been executed yet.
    fn upgrade_plan(&self) -> Result<Plan, UpgradeClientError>;

    /// Returns the upgraded client state at the specified upgrade path.
    fn upgraded_client_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<UpgradedClientStateRef<Self>, UpgradeClientError>;

    /// Returns the upgraded consensus state at the specified upgrade path.
    fn upgraded_consensus_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<UpgradedConsensusStateRef<Self>, UpgradeClientError>;
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
        client_state: UpgradedClientStateRef<Self>,
    ) -> Result<(), UpgradeClientError>;

    /// Stores the upgraded consensus state at the specified upgrade path.
    fn store_upgraded_consensus_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        consensus_state: UpgradedConsensusStateRef<Self>,
    ) -> Result<(), UpgradeClientError>;
}
