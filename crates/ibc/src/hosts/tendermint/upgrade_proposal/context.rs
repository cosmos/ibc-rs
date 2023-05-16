use alloc::boxed::Box;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics24_host::path::UpgradeClientPath;
use crate::core::{ContextError, ExecutionContext, ValidationContext};
use crate::Height;

use super::proposal::Plan;

/// Extends `ValidationContext` capability by providing required methods to
/// handle an upgrade client proposal.
pub trait UpgradeValidationContext: ValidationContext {
    /// Returns the upgrade plan that is scheduled and not have been executed yet.
    fn upgrade_plan(&self) -> Result<Plan, ContextError>;

    /// Returns the upgraded client state at the specified upgrade path.
    fn upgraded_client_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<Box<dyn ClientState>, ContextError>;

    /// Returns the upgraded consensus state at the specified upgrade path.
    fn upgraded_consensus_state(
        &self,
        upgrade_path: &UpgradeClientPath,
    ) -> Result<Box<dyn ConsensusState>, ContextError>;
}

/// Extends `ExecutionContext` capability by providing required methods to
/// handle an upgrade client proposal
pub trait UpgradeExecutionContext: ExecutionContext + UpgradeValidationContext {
    /// Schedules an upgrade based on the specified plan. If there is another Plan it should be overwritten.
    fn schedule_upgrade(&mut self, plan: Plan) -> Result<(), ContextError>;

    /// Clears the upgrade plan.
    fn clear_upgrade_plan(&mut self, plan_height: Height) -> Result<(), ContextError>;

    /// Stores the upgraded client state at the specified upgrade path.
    fn store_upgraded_client_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), ContextError>;

    /// Stores the upgraded consensus state at the specified upgrade path.
    fn store_upgraded_consensus_state(
        &mut self,
        upgrade_path: UpgradeClientPath,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), ContextError>;
}
