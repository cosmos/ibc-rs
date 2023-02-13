use crate::core::ics04_channel::events::ChannelClosed;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::prelude::*;

use crate::{
    core::{
        ics04_channel::{
            channel::{Order, State},
            error::ChannelError,
            events::TimeoutPacket,
            handler::{timeout, timeout_on_close},
        },
        ics24_host::path::CommitmentsPath,
        ics26_routing::context::ModuleId,
    },
    events::IbcEvent,
};

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) enum TimeoutMsgType {
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}

pub(super) fn timeout_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    match &timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => timeout::validate(ctx_a, msg),
        TimeoutMsgType::TimeoutOnClose(msg) => timeout_on_close::validate(ctx_a, msg),
    }?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };

    module
        .on_timeout_packet_validate(&packet, &signer)
        .map_err(ContextError::PacketError)
}

pub(super) fn timeout_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };

    let port_chan_id_on_a = (packet.port_on_a.clone(), packet.chan_on_a.clone());
    let chan_end_on_a = ctx_a.channel_end(&port_chan_id_on_a)?;

    // In all cases, this event is emitted
    ctx_a.emit_ibc_event(IbcEvent::TimeoutPacket(TimeoutPacket::new(
        packet.clone(),
        chan_end_on_a.ordering,
    )));

    // check if we're in the NO-OP case
    if ctx_a
        .get_packet_commitment(&(
            packet.port_on_a.clone(),
            packet.chan_on_a.clone(),
            packet.sequence,
        ))
        .is_err()
    {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, cb_result) = module.on_timeout_packet_execute(&packet, &signer);

    cb_result?;

    // apply state changes
    let chan_end_on_a = {
        let commitment_path = CommitmentsPath {
            port_id: packet.port_on_a.clone(),
            channel_id: packet.chan_on_a.clone(),
            sequence: packet.sequence,
        };
        ctx_a.delete_packet_commitment(commitment_path)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            let mut chan_end_on_a = chan_end_on_a;
            chan_end_on_a.state = State::Closed;
            ctx_a.store_channel(port_chan_id_on_a, chan_end_on_a.clone())?;

            chan_end_on_a
        } else {
            chan_end_on_a
        }
    };

    // emit events and logs
    {
        ctx_a.log_message("success: packet timeout".to_string());

        if let Order::Ordered = chan_end_on_a.ordering {
            let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();

            ctx_a.emit_ibc_event(IbcEvent::ChannelClosed(ChannelClosed::new(
                packet.port_on_a.clone(),
                packet.chan_on_a.clone(),
                chan_end_on_a.counterparty().port_id.clone(),
                chan_end_on_a.counterparty().channel_id.clone(),
                conn_id_on_a,
                chan_end_on_a.ordering,
            )));
        }

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::test_utils::DummyTransferModule;
    use crate::timestamp::ZERO_DURATION;
    use crate::{
        core::{
            ics03_connection::connection::ConnectionEnd,
            ics04_channel::{
                channel::{ChannelEnd, Counterparty},
                msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close,
                Version,
            },
            ics24_host::identifier::{ClientId, ConnectionId},
        },
        mock::context::MockContext,
        Height,
    };

    #[test]
    fn timeout_on_close_execute() {
        let default_context = MockContext::default();
        let height = 2;
        let timeout_timestamp = 5;

        let client_height = Height::new(0, 2).unwrap();

        let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
            height,
            timeout_timestamp,
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let data = default_context.packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b)),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Open,
            ClientId::default(),
            ConnectionCounterparty::new(
                ClientId::default(),
                Some(ConnectionId::default()),
                Default::default(),
            ),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let mut ctx = MockContext::default()
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(packet.port_on_a, packet.chan_on_a, chan_end_on_a)
            .with_packet_commitment(
                msg.packet.port_on_a.clone(),
                msg.packet.chan_on_a.clone(),
                msg.packet.sequence,
                data,
            );
        let transfer_module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        let module = DummyTransferModule::new(ctx.ibc_store_share());
        ctx.add_route(transfer_module_id.clone(), module).unwrap();

        let res = timeout_packet_execute(
            &mut ctx,
            transfer_module_id,
            TimeoutMsgType::TimeoutOnClose(msg),
        );

        assert!(res.is_ok());

        // Unordered channnels only emit one event
        assert_eq!(ctx.events.len(), 1);
        assert!(matches!(
            ctx.events.first().unwrap(),
            &IbcEvent::TimeoutPacket(_)
        ));
    }

    // TODO: test ordered channel too (ChannelClosed event also emitted)
}
