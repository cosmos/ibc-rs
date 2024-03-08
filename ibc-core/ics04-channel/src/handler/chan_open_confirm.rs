//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenConfirm`.

use ibc_core_channel_types::channel::{ChannelEnd, Counterparty, State, State as ChannelState};
use ibc_core_channel_types::error::ChannelError;
use ibc_core_channel_types::events::OpenConfirm;
use ibc_core_channel_types::msgs::MsgChannelOpenConfirm;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::types::State as ConnectionState;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{ChannelEndPath, ClientConsensusStatePath, Path};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Protobuf;

pub fn chan_open_confirm_validate<ValCtx>(
    ctx_b: &ValCtx,
    module: &dyn Module,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_b, &msg)?;

    module.on_chan_open_confirm_validate(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    Ok(())
}

pub fn chan_open_confirm_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let extras = module.on_chan_open_confirm_execute(&msg.port_id_on_b, &msg.chan_id_on_b)?;
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // state changes
    {
        let chan_end_on_b = {
            let mut chan_end_on_b = chan_end_on_b.clone();
            chan_end_on_b.set_state(State::Open);

            chan_end_on_b
        };
        ctx_b.store_channel(&chan_end_path_on_b, chan_end_on_b)?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: channel open confirm".to_string())?;

        let conn_id_on_b = chan_end_on_b.connection_hops[0].clone();
        let port_id_on_a = chan_end_on_b.counterparty().port_id.clone();
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id
            .clone()
            .ok_or(ContextError::ChannelError(ChannelError::Other {
            description:
                "internal error: ChannelEnd doesn't have a counterparty channel id in OpenConfirm"
                    .to_string(),
        }))?;

        let core_event = IbcEvent::OpenConfirmChannel(OpenConfirm::new(
            msg.port_id_on_b.clone(),
            msg.chan_id_on_b.clone(),
            port_id_on_a,
            chan_id_on_a,
            conn_id_on_b,
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

fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenConfirm) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    // Unwrap the old channel end and validate it against the message.
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Validate that the channel end is in a state where it can be confirmed.
    chan_end_on_b.verify_state_matches(&ChannelState::TryOpen)?;

    // An OPEN IBC connection running on the local (host) chain should exist.
    chan_end_on_b.verify_connection_hops_length()?;

    let conn_end_on_b = ctx_b.connection_end(&chan_end_on_b.connection_hops()[0])?;

    conn_end_on_b.verify_state_matches(&ConnectionState::Open)?;

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
        let port_id_on_a = &chan_end_on_b.counterparty().port_id;
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id()
            .ok_or(ChannelError::MissingCounterparty)?;
        let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_b.connection_hops()[0].clone(),
            },
        )?;

        let expected_chan_end_on_a = ChannelEnd::new(
            ChannelState::Open,
            *chan_end_on_b.ordering(),
            Counterparty::new(msg.port_id_on_b.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            chan_end_on_b.version.clone(),
        )?;
        let chan_end_path_on_a = ChannelEndPath::new(port_id_on_a, chan_id_on_a);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked in msg.
        client_state_of_a_on_b
            .verify_membership(
                prefix_on_a,
                &msg.proof_chan_end_on_a,
                consensus_state_of_a_on_b.root(),
                Path::ChannelEnd(chan_end_path_on_a),
                expected_chan_end_on_a.encode_vec(),
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    Ok(())
}
