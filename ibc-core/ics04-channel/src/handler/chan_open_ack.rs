//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenAck`.
use ibc_core_channel_types::channel::{ChannelEnd, Counterparty, State, State as ChannelState};
use ibc_core_channel_types::error::ChannelError;
use ibc_core_channel_types::events::OpenAck;
use ibc_core_channel_types::msgs::MsgChannelOpenAck;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::types::State as ConnectionState;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{ChannelEndPath, ClientConsensusStatePath, Path};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Protobuf;

pub fn chan_open_ack_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    module.on_chan_open_ack_validate(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;

    Ok(())
}

pub fn chan_open_ack_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let extras =
        module.on_chan_open_ack_execute(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();

            chan_end_on_a.set_state(State::Open);
            chan_end_on_a.set_version(msg.version_on_b.clone());
            chan_end_on_a.set_counterparty_channel_id(msg.chan_id_on_b.clone());

            chan_end_on_a
        };
        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel open ack".to_string())?;

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::OpenAckChannel(OpenAck::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                msg.chan_id_on_b,
                conn_id_on_a,
            ))
        };
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

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenAck) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Validate that the channel end is in a state where it can be ack.
    chan_end_on_a.verify_state_matches(&ChannelState::Init)?;

    // An OPEN IBC connection running on the local (host) chain should exist.
    chan_end_on_a.verify_connection_hops_length()?;

    let conn_end_on_a = ctx_a.connection_end(&chan_end_on_a.connection_hops()[0])?;

    conn_end_on_a.verify_state_matches(&ConnectionState::Open)?;

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_val_ctx_a = ctx_a.get_client_validation_context();
        let client_state_of_b_on_a = client_val_ctx_a.client_state(client_id_on_a)?;

        client_state_of_b_on_a
            .status(ctx_a.get_client_validation_context(), client_id_on_a)?
            .verify_is_active()?;
        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            client_id_on_a.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = &chan_end_on_a.counterparty().port_id;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;

        let expected_chan_end_on_b = ChannelEnd::new(
            ChannelState::TryOpen,
            // Note: Both ends of a channel must have the same ordering, so it's
            // fine to use A's ordering here
            *chan_end_on_a.ordering(),
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            vec![conn_id_on_b.clone()],
            msg.version_on_b.clone(),
        )?;
        let chan_end_path_on_b = ChannelEndPath::new(port_id_on_b, &msg.chan_id_on_b);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_membership(
                prefix_on_b,
                &msg.proof_chan_end_on_b,
                consensus_state_of_b_on_a.root(),
                Path::ChannelEnd(chan_end_path_on_b),
                expected_chan_end_on_b.encode_vec(),
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    Ok(())
}
