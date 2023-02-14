use crate::handler::HandlerOutputBuilder;
use crate::prelude::*;

use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::handler::dispatch as ics2_msg_dispatcher;
use crate::core::ics03_connection::handler::dispatch as ics3_msg_dispatcher;
use crate::core::ics04_channel::handler::{
    channel_callback, channel_dispatch, channel_validate, recv_packet::RecvPacketResult,
};
use crate::core::ics04_channel::handler::{
    channel_events, get_module_for_packet_msg, packet_callback as ics4_packet_callback,
    packet_dispatch as ics4_packet_msg_dispatcher,
};
use crate::core::ics04_channel::packet::PacketResult;
use crate::core::ics26_routing::context::RouterContext;
use crate::core::ics26_routing::error::RouterError;
use crate::core::ics26_routing::msgs::MsgEnvelope::{self, Channel, Client, Connection, Packet};
use crate::{events::IbcEvent, handler::HandlerOutput};

/// Result of message execution - comprises of events emitted and logs entries created during the
/// execution of a transaction message.
pub struct MsgReceipt {
    pub events: Vec<IbcEvent>,
    pub log: Vec<String>,
}

/// Mimics the DeliverTx ABCI interface, but for a single message and at a slightly lower level.
/// No need for authentication info or signature checks here.
/// Returns a vector of all events that got generated as a byproduct of processing `message`.
pub fn deliver<Ctx>(ctx: &mut Ctx, message: Any) -> Result<MsgReceipt, RouterError>
where
    Ctx: RouterContext,
{
    // Decode the proto message into a domain message, creating an ICS26 envelope.
    let envelope = decode(message)?;

    // Process the envelope, and accumulate any events that were generated.
    let HandlerOutput { log, events, .. } = dispatch(ctx, envelope)?;

    Ok(MsgReceipt { events, log })
}

/// Attempts to convert a message into a [MsgEnvelope] message
pub fn decode(message: Any) -> Result<MsgEnvelope, RouterError> {
    message.try_into()
}

/// Top-level ICS dispatch function. Routes incoming IBC messages to their corresponding module.
/// Returns a handler output with empty result of type `HandlerOutput<()>` which contains the log
/// and events produced after processing the input `msg`.
/// If this method returns an error, the runtime is expected to rollback all state modifications to
/// the `Ctx` caused by all messages from the transaction that this `msg` is a part of.
pub fn dispatch<Ctx>(ctx: &mut Ctx, msg: MsgEnvelope) -> Result<HandlerOutput<()>, RouterError>
where
    Ctx: RouterContext,
{
    let output = match msg {
        Client(msg) => {
            let handler_output =
                ics2_msg_dispatcher(ctx, msg).map_err(|e| RouterError::ContextError(e.into()))?;

            // Apply the result to the context (host chain store).
            ctx.store_client_result(handler_output.result)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            HandlerOutput::builder()
                .with_log(handler_output.log)
                .with_events(handler_output.events)
                .with_result(())
        }

        Connection(msg) => {
            let handler_output =
                ics3_msg_dispatcher(ctx, msg).map_err(|e| RouterError::ContextError(e.into()))?;

            // Apply any results to the host chain store.
            ctx.store_connection_result(handler_output.result)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            HandlerOutput::builder()
                .with_log(handler_output.log)
                .with_events(handler_output.events)
                .with_result(())
        }

        Channel(msg) => {
            let module_id =
                channel_validate(ctx, &msg).map_err(|e| RouterError::ContextError(e.into()))?;
            let dispatch_output = HandlerOutputBuilder::<()>::new();

            let (dispatch_log, mut channel_result) =
                channel_dispatch(ctx, &msg).map_err(|e| RouterError::ContextError(e.into()))?;

            // Note: `OpenInit` and `OpenTry` modify the `version` field of the `channel_result`,
            // so we must pass it mutably. We intend to clean this up with the implementation of
            // ADR 5.
            // See issue [#190](https://github.com/cosmos/ibc-rs/issues/190)
            let callback_extras = channel_callback(ctx, &module_id, &msg, &mut channel_result)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            // We need to construct events here instead of directly in the
            // `process` functions because we need to wait for the callback to
            // give us the `version` in the case of `OpenInit` and `OpenTry`.
            let dispatch_events = channel_events(
                &msg,
                channel_result.channel_id.clone(),
                channel_result.channel_end.counterparty().clone(),
                channel_result.channel_end.connection_hops[0].clone(),
                &channel_result.channel_end.version,
            );

            // Apply any results to the host chain store.
            ctx.store_channel_result(channel_result)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            dispatch_output
                .with_events(dispatch_events)
                .with_events(
                    callback_extras
                        .events
                        .into_iter()
                        .map(IbcEvent::AppModule)
                        .collect(),
                )
                .with_log(dispatch_log)
                .with_log(callback_extras.log)
                .with_result(())
        }

        Packet(msg) => {
            let module_id = get_module_for_packet_msg(ctx, &msg)
                .map_err(|e| RouterError::ContextError(e.into()))?;
            let (mut handler_builder, packet_result) = ics4_packet_msg_dispatcher(ctx, &msg)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            if matches!(packet_result, PacketResult::Recv(RecvPacketResult::NoOp)) {
                return Ok(handler_builder.with_result(()));
            }

            let cb_result = ics4_packet_callback(ctx, &module_id, &msg, &mut handler_builder);
            cb_result.map_err(|e| RouterError::ContextError(e.into()))?;

            // Apply any results to the host chain store.
            ctx.store_packet_result(packet_result)
                .map_err(|e| RouterError::ContextError(e.into()))?;

            handler_builder.with_result(())
        }
    };

    Ok(output)
}
