use crate::core::ics24_host::path::{ChannelEndPath, CommitmentPath, SeqAckPath};
use crate::prelude::*;

use crate::{
    core::{
        ics04_channel::{
            channel::Order, error::ChannelError, events::AcknowledgePacket,
            handler::acknowledgement, msgs::acknowledgement::MsgAcknowledgement,
        },
        ics26_routing::context::ModuleId,
    },
    events::IbcEvent,
};

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    acknowledgement::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    module
        .on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
        .map_err(ContextError::PacketError)
}

pub(super) fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_a =
        ChannelEndPath::new(&msg.packet.port_id_on_a, &msg.packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;
    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

    // In all cases, this event is emitted
    let event = IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        msg.packet.clone(),
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    ));
    ctx_a.emit_ibc_event(IbcEvent::Message(event.event_type()));
    ctx_a.emit_ibc_event(event);

    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
    );

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

    let (extras, cb_result) =
        module.on_acknowledgement_packet_execute(&msg.packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        let commitment_path_on_a = CommitmentPath {
            port_id: msg.packet.port_id_on_a.clone(),
            channel_id: msg.packet.chan_id_on_a.clone(),
            sequence: msg.packet.seq_on_a,
        };
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            // Note: in validation, we verified that `msg.packet.sequence == nextSeqRecv`
            // (where `nextSeqRecv` is the value in the store)
            let seq_ack_path_on_a =
                SeqAckPath::new(&msg.packet.port_id_on_a, &msg.packet.chan_id_on_a);
            ctx_a.store_next_sequence_ack(&seq_ack_path_on_a, msg.packet.seq_on_a.increment())?;
        }
    }

    // emit events and logs
    {
        ctx_a.log_message("success: packet acknowledgement".to_string());

        // Note: Acknowledgement event was emitted at the beginning

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

    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics04_channel::channel::Counterparty;
    use crate::core::ics04_channel::channel::State;
    use crate::core::ics04_channel::commitment::compute_packet_commitment;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::ChannelId;
    use crate::core::ics24_host::identifier::PortId;
    use crate::{
        applications::transfer::MODULE_ID_STR,
        core::{
            ics03_connection::{connection::ConnectionEnd, version::get_compatible_versions},
            ics04_channel::{
                channel::ChannelEnd, commitment::PacketCommitment,
                msgs::acknowledgement::test_util::get_dummy_raw_msg_acknowledgement,
            },
            ics24_host::identifier::{ClientId, ConnectionId},
        },
        events::IbcEventType,
        mock::context::MockContext,
        test_utils::DummyTransferModule,
        timestamp::ZERO_DURATION,
        Height,
    };

    struct Fixture {
        ctx: MockContext,
        module_id: ModuleId,
        msg: MsgAcknowledgement,
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

        let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
            client_height.revision_height(),
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let packet_commitment = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a_unordered = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(
                packet.port_id_on_b.clone(),
                Some(packet.chan_id_on_b.clone()),
            ),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

        let chan_end_on_a_ordered = ChannelEnd::new(
            State::Open,
            Order::Ordered,
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
            chan_end_on_a_unordered,
            chan_end_on_a_ordered,
        }
    }

    #[rstest]
    fn ack_unordered_chan_execute(fixture: Fixture) {
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

        let res = acknowledgement_packet_execute(&mut ctx, module_id, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(IbcEventType::AckPacket)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
    }

    #[rstest]
    fn ack_ordered_chan_execute(fixture: Fixture) {
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

        let res = acknowledgement_packet_execute(&mut ctx, module_id, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(IbcEventType::AckPacket)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
    }
}
