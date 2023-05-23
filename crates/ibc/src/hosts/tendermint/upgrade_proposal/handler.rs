use core::convert::Infallible;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use tendermint::abci::Event as TmEvent;
use tendermint_proto::abci::Event as ProtoEvent;

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::error::UpgradeClientError;
use crate::core::ics24_host::path::UpgradeClientPath;
use crate::hosts::tendermint::upgrade_proposal::UpgradeClientProposal;
use crate::hosts::tendermint::upgrade_proposal::UpgradeExecutionContext;
use crate::hosts::tendermint::upgrade_proposal::UpgradeProposal;

/// Handles an upgrade client proposal
///
/// It clears both IBC client and consensus states if a previous plan was set.
/// Then it will schedule an upgrade and finally set the upgraded client state
/// in upgrade store.
pub fn upgrade_client_proposal_handler<Ctx>(
    ctx: &mut Ctx,
    proposal: UpgradeProposal,
) -> Result<Vec<ProtoEvent>, UpgradeClientError>
where
    Ctx: UpgradeExecutionContext,
{
    let plan = proposal.plan;

    if ctx.upgrade_plan().is_ok() {
        ctx.clear_upgrade_plan(plan.height)?;
    }

    let mut client_state =
        TmClientState::try_from(proposal.upgraded_client_state).map_err(|e| {
            UpgradeClientError::InvalidUpgradeProposal {
                reason: e.to_string(),
            }
        })?;

    client_state.zero_custom_fields();

    ctx.schedule_upgrade(plan.clone())?;

    let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);

    ctx.store_upgraded_client_state(upgraded_client_state_path, Box::new(client_state))?;

    let event = TmEvent::from(UpgradeClientProposal::new(proposal.title, plan.height))
        .try_into()
        .map_err(|e: Infallible| UpgradeClientError::Other {
            reason: e.to_string(),
        })?;

    Ok(vec![event])
}
