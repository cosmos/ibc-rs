//! Protocol logic specific to processing ICS2 messages of type `MsgCreateClient`.

use ibc_core_client_context::client_state::{ClientStateCommon, ClientStateExecution};
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::events::CreateClient;
use ibc_core_client_types::msgs::MsgCreateClient;
use ibc_core_context::types::error::ContextError;
use ibc_core_context::types::events::{IbcEvent, MessageEvent};
use ibc_core_context::{ExecutionContext, ValidationContext};
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;

pub fn validate<Ctx>(ctx: &Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer,
    } = msg;

    ctx.validate_message_signer(&signer)?;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_state = ctx.decode_client_state(client_state)?;

    client_state.verify_consensus_state(consensus_state)?;

    let client_type = client_state.client_type();

    let client_id = ClientId::new(client_type, id_counter).map_err(|e| {
        ClientError::ClientIdentifierConstructor {
            client_type: client_state.client_type(),
            counter: id_counter,
            validation_error: e,
        }
    })?;

    if ctx.client_state(&client_id).is_ok() {
        return Err(ClientError::ClientStateAlreadyExists { client_id }.into());
    };

    Ok(())
}

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer: _,
    } = msg;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_state = ctx.decode_client_state(client_state)?;

    let client_type = client_state.client_type();

    let client_id = ClientId::new(client_type.clone(), id_counter).map_err(|e| {
        ContextError::from(ClientError::ClientIdentifierConstructor {
            client_type: client_type.clone(),
            counter: id_counter,
            validation_error: e,
        })
    })?;

    client_state.initialise(
        ctx.get_client_execution_context(),
        &client_id,
        consensus_state,
    )?;

    ctx.increase_client_counter()?;

    let event = IbcEvent::CreateClient(CreateClient::new(
        client_id.clone(),
        client_type,
        client_state.latest_height(),
    ));
    ctx.emit_ibc_event(IbcEvent::Message(MessageEvent::Client))?;
    ctx.emit_ibc_event(event)?;

    ctx.log_message(format!(
        "success: generated new client identifier: {client_id}"
    ))?;

    Ok(())
}
