//! Protocol logic specific to processing ICS2 messages of type `MsgUpdateAnyClient`.

use prost::Message;

use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation, UpdateKind,
};
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::events::{ClientMisbehaviour, UpdateClient};
use crate::core::ics02_client::msgs::MsgUpdateOrMisbehaviour;
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn validate<Ctx>(ctx: &Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx.validate_message_signer(msg.signer())?;

    let client_id = msg.client_id().clone();
    let update_kind = match msg {
        MsgUpdateOrMisbehaviour::UpdateClient(_) => UpdateKind::UpdateClient,
        MsgUpdateOrMisbehaviour::Misbehaviour(_) => UpdateKind::SubmitMisbehaviour,
    };

    // Read client state from the host chain store. The client should already exist.
    let client_state = ctx.client_state(&client_id)?;

    {
        let status = client_state.status(ctx.get_client_validation_context(), &client_id)?;
        if !status.is_active() {
            return Err(ClientError::ClientNotActive { status }.into());
        }
    }

    let client_message = msg.client_message();

    client_state.verify_client_message(
        ctx.get_client_validation_context(),
        &client_id,
        client_message,
        &update_kind,
    )?;

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx: &mut Ctx, msg: MsgUpdateOrMisbehaviour) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let client_id = msg.client_id().clone();
    let update_kind = match msg {
        MsgUpdateOrMisbehaviour::UpdateClient(_) => UpdateKind::UpdateClient,
        MsgUpdateOrMisbehaviour::Misbehaviour(_) => UpdateKind::SubmitMisbehaviour,
    };
    let client_message = msg.client_message();

    let client_state = ctx.client_state(&client_id)?;

    let found_misbehaviour = client_state.check_for_misbehaviour(
        ctx.get_client_validation_context(),
        &client_id,
        client_message.clone(),
        &update_kind,
    )?;

    if found_misbehaviour {
        client_state.update_state_on_misbehaviour(
            ctx.get_client_execution_context(),
            &client_id,
            client_message,
            &update_kind,
        )?;

        let event = IbcEvent::ClientMisbehaviour(ClientMisbehaviour::new(
            client_id,
            client_state.client_type(),
        ));
        ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
        ctx.emit_ibc_event(event)?;
    } else {
        if !matches!(update_kind, UpdateKind::UpdateClient) {
            return Err(ClientError::MisbehaviourHandlingFailure {
                reason: "misbehaviour submitted, but none found".to_string(),
            }
            .into());
        }

        let header = client_message;

        let consensus_heights = client_state.update_state(
            ctx.get_client_execution_context(),
            &client_id,
            header.clone(),
        )?;

        {
            let event = {
                let consensus_height = consensus_heights.get(0).ok_or(ClientError::Other {
                    description: "client update state returned no updated height".to_string(),
                })?;

                IbcEvent::UpdateClient(UpdateClient::new(
                    client_id,
                    client_state.client_type(),
                    *consensus_height,
                    consensus_heights,
                    header.encode_to_vec(),
                ))
            };
            ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
            ctx.emit_ibc_event(event)?;
        }
    }

    Ok(())
}
