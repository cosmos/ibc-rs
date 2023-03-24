use crate::core::ics04_channel::events::ChannelClosed;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics24_host::path::{ChannelEndPath, CommitmentPath};
use crate::prelude::*;

use crate::{
    core::{
        ics04_channel::{
            channel::{Order, State},
            error::ChannelError,
            events::TimeoutPacket,
            handler::{timeout, timeout_on_close},
        },
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
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // In all cases, this event is emitted
    let event = IbcEvent::TimeoutPacket(TimeoutPacket::new(packet.clone(), chan_end_on_a.ordering));
    ctx_a.emit_ibc_event(IbcEvent::Message(event.event_type()));
    ctx_a.emit_ibc_event(event);

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a);

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path_on_a).is_err() {
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
        let commitment_path_on_a = CommitmentPath {
            port_id: packet.port_id_on_a.clone(),
            channel_id: packet.chan_id_on_a.clone(),
            sequence: packet.seq_on_a,
        };
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            let mut chan_end_on_a = chan_end_on_a;
            chan_end_on_a.state = State::Closed;
            ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a.clone())?;

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

            let event = IbcEvent::ChannelClosed(ChannelClosed::new(
                packet.port_id_on_a.clone(),
                packet.chan_id_on_a.clone(),
                chan_end_on_a.counterparty().port_id.clone(),
                chan_end_on_a.counterparty().channel_id.clone(),
                conn_id_on_a,
                chan_end_on_a.ordering,
            ));
            ctx_a.emit_ibc_event(IbcEvent::Message(event.event_type()));
            ctx_a.emit_ibc_event(event);
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

    use rstest::*;

    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::commitment::compute_packet_commitment;
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics24_host::identifier::ChannelId;
    use crate::core::ics24_host::identifier::PortId;
    use crate::test_utils::DummyTransferModule;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;
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
        events::IbcEventType,
        mock::context::MockContext,
    };

    struct Fixture {
        ctx: MockContext,
        module_id: ModuleId,
        msg: MsgTimeoutOnClose,
        packet_commitment: PacketCommitment,
        conn_end_on_a: ConnectionEnd,
        chan_end_on_a_ordered: ChannelEnd,
        chan_end_on_a_unordered: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let client_height = Height::new(0, 2).unwrap();
        let mut ctx = MockContext::default().with_client(&ClientId::default(), client_height);

        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        let module = DummyTransferModule::new();
        ctx.add_route(module_id.clone(), module).unwrap();

        let height = 2;
        let timeout_timestamp = 5;

        let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
            height,
            timeout_timestamp,
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let packet_commitment = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a_ordered = ChannelEnd::new(
            State::Open,
            Order::Ordered,
            Counterparty::new(
                packet.port_id_on_b.clone(),
                Some(packet.chan_id_on_b.clone()),
            ),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

        let chan_end_on_a_unordered = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(packet.port_id_on_b.clone(), Some(packet.chan_id_on_b)),
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

        Fixture {
            ctx,
            module_id,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_ordered,
            chan_end_on_a_unordered,
        }
    }

    #[rstest]
    fn timeout_unordered_chan_execute(fixture: Fixture) {
        let Fixture {
            ctx,
            module_id,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_unordered,
            ..
        } = fixture;
        let mut ctx = ctx
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_packet_commitment(
                msg.packet.port_id_on_a.clone(),
                msg.packet.chan_id_on_a.clone(),
                msg.packet.seq_on_a,
                packet_commitment,
            );

        let res = timeout_packet_execute(&mut ctx, module_id, TimeoutMsgType::TimeoutOnClose(msg));

        assert!(res.is_ok());

        // Unordered channels only emit one event
        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(IbcEventType::Timeout)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::TimeoutPacket(_)));
    }

    #[rstest]
    fn timeout_ordered_chan_execute(fixture: Fixture) {
        let Fixture {
            ctx,
            module_id,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_ordered,
            ..
        } = fixture;
        let mut ctx = ctx
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_ordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_packet_commitment(
                msg.packet.port_id_on_a.clone(),
                msg.packet.chan_id_on_a.clone(),
                msg.packet.seq_on_a,
                packet_commitment,
            );

        let res = timeout_packet_execute(&mut ctx, module_id, TimeoutMsgType::TimeoutOnClose(msg));

        assert!(res.is_ok());

        // Ordered channels emit 2 events
        assert_eq!(ctx.events.len(), 4);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(IbcEventType::Timeout)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::TimeoutPacket(_)));
        assert!(matches!(
            ctx.events[2],
            IbcEvent::Message(IbcEventType::ChannelClosed)
        ));
        assert!(matches!(ctx.events[3], IbcEvent::ChannelClosed(_)));
    }
}
