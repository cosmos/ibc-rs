//! Protocol logic specific to processing ICS2 messages of type `MsgCreateClient`.

use ibc_core_client_context::prelude::*;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::events::CreateClient;
use ibc_core_client_types::msgs::MsgCreateClient;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::{ClientStateMut, ClientStateRef, ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;

pub fn validate<Ctx>(ctx: &Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
    <ClientStateRef<Ctx> as TryFrom<Any>>::Error: Into<ClientError>,
{
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer,
    } = msg;

    ctx.validate_message_signer(&signer)?;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_val_ctx = ctx.get_client_validation_context();

    let client_state = ClientStateRef::<Ctx>::try_from(client_state).map_err(Into::into)?;

    let client_id = client_state.client_type().build_client_id(id_counter);

    let status = client_state.status(client_val_ctx, &client_id)?;

    if status.is_frozen() {
        return Err(ClientError::ClientFrozen {
            description: "the client is frozen".to_string(),
        }
        .into());
    };

    client_state.verify_consensus_state(consensus_state)?;

    if client_val_ctx.client_state(&client_id).is_ok() {
        return Err(ClientError::ClientStateAlreadyExists { client_id }.into());
    };

    Ok(())
}

pub fn execute<Ctx>(ctx: &mut Ctx, msg: MsgCreateClient) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
    <ClientStateMut<Ctx> as TryFrom<Any>>::Error: Into<ClientError>,
{
    let MsgCreateClient {
        client_state,
        consensus_state,
        signer: _,
    } = msg;

    // Construct this client's identifier
    let id_counter = ctx.client_counter()?;

    let client_exec_ctx = ctx.get_client_execution_context();

    let client_state = ClientStateMut::<Ctx>::try_from(client_state).map_err(Into::into)?;

    let client_type = client_state.client_type();
    let client_id = client_type.build_client_id(id_counter);

    client_state.initialise(client_exec_ctx, &client_id, consensus_state)?;

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
