//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use ibc_core_channel_types::channel::{ChannelEnd, Counterparty, State};
use ibc_core_channel_types::events::OpenInit;
use ibc_core_channel_types::msgs::MsgChannelOpenInit;
use ibc_core_client::context::prelude::*;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::identifiers::ChannelId;
use ibc_core_host::types::path::{ChannelEndPath, SeqAckPath, SeqRecvPath, SeqSendPath};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;

pub fn chan_open_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);

    module.on_chan_open_init_validate(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    Ok(())
}

pub fn chan_open_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);
    let (extras, version) = module.on_chan_open_init_execute(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    let conn_id_on_a = msg.connection_hops_on_a[0].clone();

    // state changes
    {
        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            msg.connection_hops_on_a.clone(),
            msg.version_proposal.clone(),
        )?;
        let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;

        ctx_a.increase_channel_counter()?;

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_send(&seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_recv(&seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_ack(&seq_ack_path, 1.into())?;
    }

    // emit events and logs
    {
        ctx_a.log_message(format!(
            "success: channel open init with channel identifier: {chan_id_on_a}"
        ))?;
        let core_event = IbcEvent::OpenInitChannel(OpenInit::new(
            msg.port_id_on_a.clone(),
            chan_id_on_a.clone(),
            msg.port_id_on_b,
            conn_id_on_a,
            version,
        ));
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_a.emit_ibc_event(core_event)?;

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event))?;
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenInit) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    msg.verify_connection_hops_length()?;
    // An IBC connection running on the local (host) chain should exist.
    let conn_end_on_a = ctx_a.connection_end(&msg.connection_hops_on_a[0])?;

    // Note: Not needed check if the connection end is OPEN. Optimistic channel handshake is allowed.

    let client_id_on_a = conn_end_on_a.client_id();
    let client_val_ctx_a = ctx_a.get_client_validation_context();
    let client_state_of_b_on_a = client_val_ctx_a.client_state(client_id_on_a)?;

    client_state_of_b_on_a
        .status(ctx_a.get_client_validation_context(), client_id_on_a)?
        .verify_is_active()?;

    let conn_version = conn_end_on_a.versions();

    conn_version[0].verify_feature_supported(msg.ordering.to_string())?;

    Ok(())
}
