//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenTry`.

use core::str::FromStr;

use ibc_eureka_core_channel_types::channel::{ChannelEnd, Counterparty, State as ChannelState};
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::OpenTry;
use ibc_eureka_core_channel_types::msgs::MsgChannelOpenTry;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_connection::types::error::ConnectionError;
use ibc_eureka_core_connection::types::State as ConnectionState;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::identifiers::ChannelId;
use ibc_eureka_core_host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, Path, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_eureka_core_host::{ExecutionContext, ValidationContext};
use ibc_eureka_core_router::module::Module;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Protobuf;

pub fn chan_open_try_validate<ValCtx>(
    ctx_b: &ValCtx,
    module: &dyn Module,
    msg: MsgChannelOpenTry,
) -> Result<(), ChannelError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_b, &msg)?;

    // todo(rano): hack
    let chan_id_on_b = ChannelId::from_str("00-dummy-0")?;

    module.on_chan_open_try_validate(
        msg.ordering,
        &msg.port_id_on_b,
        &chan_id_on_b,
        &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        &msg.version_supported_on_a,
    )?;

    Ok(())
}

pub fn chan_open_try_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgChannelOpenTry,
) -> Result<(), ChannelError>
where
    ExecCtx: ExecutionContext,
{
    // todo(rano): hack
    let chan_id_on_b = ChannelId::from_str("00-dummy-0")?;
    let (extras, version) = module.on_chan_open_try_execute(
        msg.ordering,
        &msg.port_id_on_b,
        &chan_id_on_b,
        &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        &msg.version_supported_on_a,
    )?;

    // state changes
    {
        let chan_end_on_b = ChannelEnd::new(
            ChannelState::TryOpen,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            version.clone(),
        )?;

        let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_channel(&chan_end_path_on_b, chan_end_on_b)?;
        ctx_b.increase_channel_counter()?;

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_send(&seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_recv(&seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_ack(&seq_ack_path, 1.into())?;
    }

    // emit events and logs
    {
        ctx_b.log_message(format!(
            "success: channel open try with channel identifier: {chan_id_on_b}"
        ))?;

        let core_event = IbcEvent::OpenTryChannel(OpenTry::new(
            msg.port_id_on_b.clone(),
            chan_id_on_b.clone(),
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            version,
        ));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_b.emit_ibc_event(core_event)?;

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::Module(module_event))?;
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenTry) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_val_ctx_b = ctx_b.get_client_validation_context();
        let client_state_of_a_on_b = client_val_ctx_b.client_state(client_id_on_b)?;

        client_state_of_a_on_b
            .status(ctx_b.get_client_validation_context(), client_id_on_b)?
            .verify_is_active()?;

        client_state_of_a_on_b.validate_proof_height(msg.proof_height_on_a)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            client_id_on_b.clone(),
            msg.proof_height_on_a.revision_number(),
            msg.proof_height_on_a.revision_height(),
        );
        let consensus_state_of_a_on_b =
            client_val_ctx_b.consensus_state(&client_cons_state_path_on_b)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let port_id_on_a = msg.port_id_on_a.clone();
        let chan_id_on_a = msg.chan_id_on_a.clone();
        let conn_id_on_a = conn_end_on_b
            .counterparty()
            .connection_id()
            .ok_or(ConnectionError::MissingCounterparty)?;

        let expected_chan_end_on_a = ChannelEnd::new(
            ChannelState::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            msg.version_supported_on_a.clone(),
        )?;
        let chan_end_path_on_a = ChannelEndPath::new(&port_id_on_a, &chan_id_on_a);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_a_on_b.verify_membership(
            prefix_on_a,
            &msg.proof_chan_end_on_a,
            consensus_state_of_a_on_b.root(),
            Path::ChannelEnd(chan_end_path_on_a),
            expected_chan_end_on_a.encode_vec(),
        )?;
    }

    Ok(())
}
