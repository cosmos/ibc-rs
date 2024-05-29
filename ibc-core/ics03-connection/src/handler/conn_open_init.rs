//! Protocol logic specific to ICS3 messages of type `MsgConnectionOpenInit`.
use ibc_core_client::context::prelude::*;
use ibc_core_connection_types::events::OpenInit;
use ibc_core_connection_types::msgs::MsgConnectionOpenInit;
use ibc_core_connection_types::{ConnectionEnd, Counterparty, State};
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::identifiers::ConnectionId;
use ibc_core_host::types::path::{ClientConnectionPath, ConnectionPath};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;

pub fn validate<Ctx>(ctx_a: &Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let client_val_ctx_a = ctx_a.get_client_validation_context();

    // An IBC client running on the local (host) chain should exist.
    let client_state_of_b_on_a = client_val_ctx_a.client_state(&msg.client_id_on_a)?;

    client_state_of_b_on_a
        .status(client_val_ctx_a, &msg.client_id_on_a)?
        .verify_is_active()?;

    if let Some(version) = msg.version {
        version.verify_is_supported(&ctx_a.get_compatible_versions())?;
    }

    Ok(())
}

pub fn execute<Ctx>(ctx_a: &mut Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let versions = if let Some(version) = msg.version {
        version.verify_is_supported(&ctx_a.get_compatible_versions())?;
        vec![version]
    } else {
        ctx_a.get_compatible_versions()
    };

    let conn_end_on_a = ConnectionEnd::new(
        State::Init,
        msg.client_id_on_a.clone(),
        Counterparty::new(
            msg.counterparty.client_id().clone(),
            None,
            msg.counterparty.prefix().clone(),
        ),
        versions,
        msg.delay_period,
    )?;

    // Construct the identifier for the new connection.
    let conn_id_on_a = ConnectionId::new(ctx_a.connection_counter()?);

    ctx_a.log_message(format!(
        "success: conn_open_init: generated new connection identifier: {conn_id_on_a}"
    ))?;

    {
        let client_id_on_b = msg.counterparty.client_id().clone();

        let event = IbcEvent::OpenInitConnection(OpenInit::new(
            conn_id_on_a.clone(),
            msg.client_id_on_a.clone(),
            client_id_on_b,
        ));
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Connection))?;
        ctx_a.emit_ibc_event(event)?;
    }

    ctx_a.increase_connection_counter()?;
    ctx_a.store_connection_to_client(
        &ClientConnectionPath::new(msg.client_id_on_a),
        conn_id_on_a.clone(),
    )?;
    ctx_a.store_connection(&ConnectionPath::new(&conn_id_on_a), conn_end_on_a)?;

    Ok(())
}
