//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use ibc_core_client_context::prelude::*;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::events::UpgradeClient;
use ibc_core_client_types::msgs::MsgUpgradeClient;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;

pub fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgUpgradeClient {
        client_id, signer, ..
    } = msg;

    ctx.validate_message_signer(&signer)?;

    let client_val_ctx = ctx.get_client_validation_context();

    // Read the current latest client state from the host chain store.
    let old_client_state = client_val_ctx.client_state(&client_id)?;

    // Check if the client is active.
    old_client_state
        .status(client_val_ctx, &client_id)?
        .verify_is_active()?;

    // Read the latest consensus state from the host chain store.
    let old_client_cons_state_path = ClientConsensusStatePath::new(
        client_id.clone(),
        old_client_state.latest_height().revision_number(),
        old_client_state.latest_height().revision_height(),
    );
    let old_consensus_state = client_val_ctx
        .consensus_state(&old_client_cons_state_path)
        .map_err(|_| ClientError::ConsensusStateNotFound {
            client_id,
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

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgUpgradeClient { client_id, .. } = msg;

    let client_exec_ctx = ctx.get_client_execution_context();

    let old_client_state = client_exec_ctx.client_state(&client_id)?;

    let latest_height = old_client_state.update_state_on_upgrade(
        client_exec_ctx,
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
