//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation,
};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::UpgradeClient;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient {
        client_id, signer, ..
    } = msg;

    ctx.validate_message_signer(&signer)?;

    // Read the current latest client state from the host chain store.
    let old_client_state = ctx.client_state(&client_id)?;

    // Check if the client is active.
    {
        let status = old_client_state.status(ctx.get_client_validation_context(), &client_id)?;
        if !status.is_active() {
            return Err(ClientError::ClientNotActive { status }.into());
        }
    }

    // Read the latest consensus state from the host chain store.
    let old_client_cons_state_path =
        ClientConsensusStatePath::new(&client_id, &old_client_state.latest_height());
    let old_consensus_state = ctx
        .consensus_state(&old_client_cons_state_path)
        .map_err(|_| ClientError::ConsensusStateNotFound {
            client_id: client_id.clone(),
            height: old_client_state.latest_height(),
        })?;

    // Validate the upgraded client state and consensus state and verify proofs against the root
    old_client_state.verify_upgrade_client(
        msg.upgraded_client_state.clone(),
        msg.upgraded_consensus_state,
        msg.proof_upgrade_client,
        msg.proof_upgrade_consensus_state,
        old_consensus_state.root(),
    )?;

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    let old_client_state = ctx.client_state(&client_id)?;

    let latest_height = old_client_state.update_state_on_upgrade(
        ctx.get_client_execution_context(),
        &client_id,
        msg.upgraded_client_state.clone(),
        msg.upgraded_consensus_state,
    )?;

    let event = IbcEvent::UpgradeClient(UpgradeClient::new(
        client_id,
        old_client_state.client_type(),
        latest_height,
    ));
    ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
    ctx.emit_ibc_event(event)?;

    Ok(())
}
