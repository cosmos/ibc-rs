//! This module implements the processing logic for ICS4 (channel) messages.
use crate::events::{IbcEvent, ModuleEvent};
use crate::prelude::*;

use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::msgs::ChannelMsg;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics04_channel::{msgs::PacketMsg, packet::PacketResult};
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::core::ics26_routing::context::{
    Acknowledgement, ModuleId, ModuleOutputBuilder, OnRecvPacketAck, Router, RouterContext,
};
use crate::handler::{HandlerOutput, HandlerOutputBuilder};

use super::channel::Counterparty;
use super::events::{CloseConfirm, CloseInit, OpenAck, OpenConfirm, OpenInit, OpenTry};
use super::Version;

pub mod acknowledgement;
pub mod chan_close_confirm;
pub mod chan_close_init;
pub mod chan_open_ack;
pub mod chan_open_confirm;
pub mod chan_open_init;
pub mod chan_open_try;
pub mod recv_packet;
pub mod send_packet;
pub mod timeout;
pub mod timeout_on_close;
pub mod verify;
pub mod write_acknowledgement;

/// Defines the possible states of a channel identifier in a `ChannelResult`.
#[derive(Clone, Debug)]
pub enum ChannelIdState {
    /// Specifies that the channel handshake handler allocated a new channel identifier. This
    /// happens during the processing of either the `MsgChannelOpenInit` or `MsgChannelOpenTry`.
    Generated,

    /// Specifies that the handler reused a previously-allocated channel identifier.
    Reused,
}

#[derive(Clone, Debug)]
pub struct ChannelResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub channel_id_state: ChannelIdState,
    pub channel_end: ChannelEnd,
}

pub struct ModuleExtras {
    pub events: Vec<ModuleEvent>,
    pub log: Vec<String>,
}

impl ModuleExtras {
    pub fn empty() -> Self {
        ModuleExtras {
            events: Vec::new(),
            log: Vec::new(),
        }
    }
}

pub(crate) fn channel_validate<Ctx>(ctx: &Ctx, msg: &ChannelMsg) -> Result<ModuleId, ChannelError>
where
    Ctx: RouterContext,
{
    let module_id = msg.lookup_module(ctx)?;
    if ctx.router().has_route(&module_id) {
        Ok(module_id)
    } else {
        Err(ChannelError::RouteNotFound)
    }
}

/// General entry point for processing any type of message related to the ICS4 channel open and
/// channel close handshake protocols.
pub(crate) fn channel_dispatch<Ctx>(
    ctx: &Ctx,
    msg: &ChannelMsg,
) -> Result<(Vec<String>, ChannelResult), ChannelError>
where
    Ctx: ChannelReader,
{
    let output = match msg {
        ChannelMsg::OpenInit(msg) => chan_open_init::process(ctx, msg),
        ChannelMsg::OpenTry(msg) => chan_open_try::process(ctx, msg),
        ChannelMsg::OpenAck(msg) => chan_open_ack::process(ctx, msg),
        ChannelMsg::OpenConfirm(msg) => chan_open_confirm::process(ctx, msg),
        ChannelMsg::CloseInit(msg) => chan_close_init::process(ctx, msg),
        ChannelMsg::CloseConfirm(msg) => chan_close_confirm::process(ctx, msg),
    }?;

    let HandlerOutput { result, log, .. } = output;
    Ok((log, result))
}

pub(crate) fn channel_callback<Ctx>(
    ctx: &mut Ctx,
    module_id: &ModuleId,
    msg: &ChannelMsg,
    result: &mut ChannelResult,
) -> Result<ModuleExtras, ChannelError>
where
    Ctx: RouterContext,
{
    let cb = ctx
        .router_mut()
        .get_route_mut(module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    match msg {
        ChannelMsg::OpenInit(msg) => {
            let (extras, version) = cb.on_chan_open_init(
                msg.chan_end_on_a.ordering,
                &msg.chan_end_on_a.connection_hops,
                &msg.port_id_on_a,
                &result.channel_id,
                msg.chan_end_on_a.counterparty(),
                &msg.chan_end_on_a.version,
            )?;
            result.channel_end.version = version;

            Ok(extras)
        }
        ChannelMsg::OpenTry(msg) => {
            let (extras, version) = cb.on_chan_open_try(
                msg.chan_end_on_b.ordering,
                &msg.chan_end_on_b.connection_hops,
                &msg.port_id_on_b,
                &result.channel_id,
                msg.chan_end_on_b.counterparty(),
                &msg.version_on_a,
            )?;
            result.channel_end.version = version;

            Ok(extras)
        }
        ChannelMsg::OpenAck(msg) => {
            cb.on_chan_open_ack(&msg.port_id_on_a, &result.channel_id, &msg.version_on_b)
        }
        ChannelMsg::OpenConfirm(msg) => {
            cb.on_chan_open_confirm(&msg.port_id_on_b, &result.channel_id)
        }
        ChannelMsg::CloseInit(msg) => cb.on_chan_close_init(&msg.port_id_on_a, &result.channel_id),
        ChannelMsg::CloseConfirm(msg) => {
            cb.on_chan_close_confirm(&msg.port_id_on_b, &result.channel_id)
        }
    }
}

/// Constructs the proper channel event
pub(crate) fn channel_events(
    msg: &ChannelMsg,
    channel_id: ChannelId,
    counterparty: Counterparty,
    connection_id: ConnectionId,
    version: &Version,
) -> Vec<IbcEvent> {
    let event = match msg {
        ChannelMsg::OpenInit(msg) => IbcEvent::OpenInitChannel(OpenInit::new(
            msg.port_id_on_a.clone(),
            channel_id,
            counterparty.port_id,
            connection_id,
            version.clone(),
        )),
        ChannelMsg::OpenTry(msg) => IbcEvent::OpenTryChannel(OpenTry::new(
            msg.port_id_on_b.clone(),
            channel_id,
            counterparty.port_id,
            counterparty
                .channel_id
                .expect("counterparty channel id must exist after channel open try"),
            connection_id,
            version.clone(),
        )),
        ChannelMsg::OpenAck(msg) => IbcEvent::OpenAckChannel(OpenAck::new(
            msg.port_id_on_a.clone(),
            channel_id,
            counterparty.port_id,
            counterparty
                .channel_id
                .expect("counterparty channel id must exist after channel open ack"),
            connection_id,
        )),
        ChannelMsg::OpenConfirm(msg) => IbcEvent::OpenConfirmChannel(OpenConfirm::new(
            msg.port_id_on_b.clone(),
            channel_id,
            counterparty.port_id,
            counterparty
                .channel_id
                .expect("counterparty channel id must exist after channel open confirm"),
            connection_id,
        )),
        ChannelMsg::CloseInit(msg) => IbcEvent::CloseInitChannel(CloseInit::new(
            msg.port_id_on_a.clone(),
            channel_id,
            counterparty.port_id,
            counterparty
                .channel_id
                .expect("counterparty channel id must exist after channel open ack"),
            connection_id,
        )),
        ChannelMsg::CloseConfirm(msg) => IbcEvent::CloseConfirmChannel(CloseConfirm::new(
            msg.port_id_on_b.clone(),
            channel_id,
            counterparty.port_id,
            counterparty
                .channel_id
                .expect("counterparty channel id must exist after channel open ack"),
            connection_id,
        )),
    };

    vec![event]
}

pub(crate) fn get_module_for_packet_msg<Ctx>(
    ctx: &Ctx,
    msg: &PacketMsg,
) -> Result<ModuleId, ChannelError>
where
    Ctx: RouterContext,
{
    let module_id = msg.lookup_module(ctx)?;
    if ctx.router().has_route(&module_id) {
        Ok(module_id)
    } else {
        Err(ChannelError::RouteNotFound)
    }
}

/// Dispatcher for processing any type of message related to the ICS4 packet protocols.
pub(crate) fn packet_dispatch<Ctx>(
    ctx: &Ctx,
    msg: &PacketMsg,
) -> Result<(HandlerOutputBuilder<()>, PacketResult), PacketError>
where
    Ctx: ChannelReader,
{
    let output = match msg {
        PacketMsg::Recv(msg) => recv_packet::process(ctx, msg),
        PacketMsg::Ack(msg) => acknowledgement::process(ctx, msg),
        PacketMsg::Timeout(msg) => timeout::process(ctx, msg),
        PacketMsg::TimeoutOnClose(msg) => timeout_on_close::process(ctx, msg),
    }?;
    let HandlerOutput {
        result,
        log,
        events,
    } = output;
    let builder = HandlerOutput::builder().with_log(log).with_events(events);
    Ok((builder, result))
}

pub(crate) fn packet_callback<Ctx>(
    ctx: &mut Ctx,
    module_id: &ModuleId,
    msg: &PacketMsg,
    output: &mut HandlerOutputBuilder<()>,
) -> Result<(), PacketError>
where
    Ctx: RouterContext,
{
    let mut module_output = ModuleOutputBuilder::new();
    let mut core_output = HandlerOutputBuilder::new();

    let result = do_packet_callback(ctx, module_id, msg, &mut module_output, &mut core_output);
    output.merge(module_output);
    output.merge(core_output);

    result
}

fn do_packet_callback(
    ctx: &mut impl RouterContext,
    module_id: &ModuleId,
    msg: &PacketMsg,
    module_output: &mut ModuleOutputBuilder,
    core_output: &mut HandlerOutputBuilder<()>,
) -> Result<(), PacketError> {
    let cb = ctx
        .router_mut()
        .get_route_mut(module_id)
        .ok_or(PacketError::RouteNotFound)?;

    match msg {
        PacketMsg::Recv(msg) => {
            let result = cb.on_recv_packet(module_output, &msg.packet, &msg.signer);
            match result {
                OnRecvPacketAck::Nil(write_fn) => {
                    write_fn(cb.as_any_mut()).map_err(|e| PacketError::AppModule { description: e })
                }
                OnRecvPacketAck::Successful(ack, write_fn) => {
                    write_fn(cb.as_any_mut())
                        .map_err(|e| PacketError::AppModule { description: e })?;

                    process_write_ack(ctx, msg.packet.clone(), ack.as_ref(), core_output)
                }
                OnRecvPacketAck::Failed(ack) => {
                    process_write_ack(ctx, msg.packet.clone(), ack.as_ref(), core_output)
                }
            }
        }
        PacketMsg::Ack(msg) => cb.on_acknowledgement_packet(
            module_output,
            &msg.packet,
            &msg.acknowledgement,
            &msg.signer,
        ),
        PacketMsg::Timeout(msg) => cb.on_timeout_packet(module_output, &msg.packet, &msg.signer),
        PacketMsg::TimeoutOnClose(msg) => {
            cb.on_timeout_packet(module_output, &msg.packet, &msg.signer)
        }
    }
}

fn process_write_ack(
    ctx: &mut impl RouterContext,
    packet: Packet,
    acknowledgement: &dyn Acknowledgement,
    core_output: &mut HandlerOutputBuilder<()>,
) -> Result<(), PacketError> {
    let HandlerOutput {
        result,
        log,
        events,
    } = write_acknowledgement::process(ctx, packet, acknowledgement.as_ref().to_vec().into())?;

    // store write ack result
    ctx.store_packet_result(result)?;

    core_output.merge_output(
        HandlerOutput::builder()
            .with_log(log)
            .with_events(events)
            .with_result(()),
    );

    Ok(())
}
