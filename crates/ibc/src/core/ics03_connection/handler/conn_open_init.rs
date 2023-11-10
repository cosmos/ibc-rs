//! Protocol logic specific to ICS3 messages of type `MsgConnectionOpenInit`.
use crate::core::context::ContextError;
use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::ClientStateValidation;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::events::OpenInit;
use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::ics24_host::path::{ClientConnectionPath, ConnectionPath};
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn validate<Ctx>(ctx_a: &Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    // An IBC client running on the local (host) chain should exist.
    let client_state_of_b_on_a = ctx_a.client_state(&msg.client_id_on_a)?;

    {
        let status = client_state_of_b_on_a
            .status(ctx_a.get_client_validation_context(), &msg.client_id_on_a)?;
        if !status.is_active() {
            return Err(ClientError::ClientNotActive { status }.into());
        }
    }

    if let Some(version) = msg.version {
        version.verify_is_supported(&ctx_a.get_compatible_versions())?;
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx_a: &mut Ctx, msg: MsgConnectionOpenInit) -> Result<(), ContextError>
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
        &ClientConnectionPath::new(&msg.client_id_on_a),
        conn_id_on_a.clone(),
    )?;
    ctx_a.store_connection(&ConnectionPath::new(&conn_id_on_a), conn_end_on_a)?;

    Ok(())
}
