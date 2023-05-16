use alloc::boxed::Box;

use super::context::UpgradeExecutionContext;
use super::proposal::UpgradeProposal;
use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::events::IbcEvent;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::events::UpgradeClientProposal;
use crate::core::ics24_host::path::UpgradeClientPath;
use crate::core::ContextError;
use crate::Height;

pub fn upgrade_proposal_handler<Ctx>(
    ctx: &mut Ctx,
    proposal: UpgradeProposal,
) -> Result<(), ContextError>
where
    Ctx: UpgradeExecutionContext,
{
    let mut client_state = TmClientState::try_from(proposal.upgraded_client_state)?;

    client_state.zero_custom_fields();

    ctx.schedule_upgrade(proposal.plan.clone())?;

    let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(proposal.plan.height);

    ctx.store_upgraded_client_state(upgraded_client_state_path, Box::new(client_state))?;

    let plan_height = Height::new(ctx.host_height()?.revision_number(), proposal.plan.height)?;
    ctx.emit_ibc_event(IbcEvent::UpgradeClientProposal(UpgradeClientProposal::new(
        proposal.title,
        plan_height,
    )));

    Ok(())
}
