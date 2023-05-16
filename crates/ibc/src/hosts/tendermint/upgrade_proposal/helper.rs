use alloc::string::ToString;

use super::context::UpgradeExecutionContext;
use super::proposal::Plan;
use crate::core::events::IbcEvent;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpgradeChain;
use crate::core::ics24_host::path::UpgradeClientPath;
use crate::core::ContextError;
use crate::hosts::tendermint::UPGRADE_STORE_KEY;
use crate::Height;

// Schedules an upgrade based on the specified plan. If there is another Plan
// already scheduled, it will cancel and overwrite it.
pub fn schedule_upgrade<Ctx>(ctx: &mut Ctx, plan: Plan) -> Result<(), ContextError>
where
    Ctx: UpgradeExecutionContext,
{
    // NOTE: allow for the possibility of chains to schedule upgrades in begin block of the same block
    // as a strategy for emergency hard fork recoveries
    let host_height = ctx.host_height()?;
    if plan.height < host_height.revision_height() {
        return Err(ClientError::InvalidUpgradeProposal {
            reason: "upgrade plan height is in the past".to_string(),
        })
        .map_err(ContextError::from);
    }

    let plan_height = Height::new(host_height.revision_number(), plan.height)?;

    if ctx.upgrade_plan().is_ok() {
        ctx.clear_upgrade_plan(plan_height)?;
    }

    if host_height.revision_height() == plan.height - 1 {
        let upgraded_consensus_state = ctx.host_consensus_state(&host_height)?;
        let upgraded_cons_state_path = UpgradeClientPath::UpgradedClientConsensusState(plan.height);

        ctx.store_upgraded_consensus_state(upgraded_cons_state_path, upgraded_consensus_state)?;

        ctx.emit_ibc_event(IbcEvent::UpgradeChain(UpgradeChain::new(
            plan_height,
            UPGRADE_STORE_KEY.to_string(),
        )))
    }
    Ok(())
}

// Called in the begin block of the last block of the chain that will commit the upgrade plan.
pub fn begin_blocker<Ctx>(ctx: &mut Ctx) -> Result<(), ContextError>
where
    Ctx: UpgradeExecutionContext,
{
    if let Ok(plan) = ctx.upgrade_plan() {
        // Checks if for the stored plan the upgraded client state is already set.
        let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);
        ctx.upgraded_client_state(&upgraded_client_state_path)?;

        // Once we are at the last block this chain will commit, set the upgraded consensus state
        // so that IBC clients can use the last NextValidatorsHash as a trusted kernel for verifying
        // headers on the next version of the chain.
        // Set the time to the last block time of the current chain.
        // In order for a client to upgrade successfully, the first block of the new chain must be committed
        // within the trusting period of the last block time on this chain.
        let host_height = ctx.host_height()?;
        if host_height.revision_height() == plan.height - 1 {
            let upgraded_consensus_state = ctx.host_consensus_state(&host_height)?;
            let upgraded_cons_state_path =
                UpgradeClientPath::UpgradedClientConsensusState(plan.height);
            ctx.store_upgraded_consensus_state(upgraded_cons_state_path, upgraded_consensus_state)?;

            let plan_height = Height::new(host_height.revision_number(), plan.height)?;
            ctx.emit_ibc_event(IbcEvent::UpgradeChain(UpgradeChain::new(
                plan_height,
                UPGRADE_STORE_KEY.to_string(),
            )))
        }
    }
    Ok(())
}
