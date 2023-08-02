use crate::prelude::*;

use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::ClientStateCommon;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics03_connection::delay::verify_conn_delay_passed;
use crate::core::ics04_channel::channel::{Counterparty, Order, State as ChannelState};
use crate::core::ics04_channel::commitment::{compute_ack_commitment, compute_packet_commitment};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::events::{ReceivePacket, WriteAcknowledgement};
use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::core::ics04_channel::packet::Receipt;
use crate::core::ics24_host::path::Path;
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
};
use crate::core::router::Module;
use crate::core::timestamp::Expiry;
use crate::core::{ContextError, ExecutionContext, ValidationContext};

pub(crate) fn recv_packet_validate<ValCtx>(
    ctx_b: &ValCtx,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    // Note: this contains the validation for `write_acknowledgement` as well.
    validate(ctx_b, &msg)

    // nothing to validate with the module, since `onRecvPacket` cannot fail.
    // If any error occurs, then an "error acknowledgement" must be returned.
}

pub(crate) fn recv_packet_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_b =
        ChannelEndPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let packet = &msg.packet;
                let receipt_path_on_b =
                    ReceiptPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);
                ctx_b.get_packet_receipt(&receipt_path_on_b).is_ok()
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                msg.packet.seq_on_a < next_seq_recv
            }
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        match chan_end_on_b.ordering {
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath {
                    port_id: msg.packet.port_id_on_b.clone(),
                    channel_id: msg.packet.chan_id_on_b.clone(),
                    sequence: msg.packet.seq_on_a,
                };

                ctx_b.store_packet_receipt(&receipt_path_on_b, Receipt::Ok)?;
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
                ctx_b.store_next_sequence_recv(&seq_recv_path_on_b, next_seq_recv.increment())?;
            }
            _ => {}
        }
        let ack_path_on_b = AckPath::new(
            &msg.packet.port_id_on_b,
            &msg.packet.chan_id_on_b,
            msg.packet.seq_on_a,
        );
        // `writeAcknowledgement` handler state changes
        ctx_b.store_packet_acknowledgement(
            &ack_path_on_b,
            compute_ack_commitment(&acknowledgement),
        )?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: packet receive".to_string());
        ctx_b.log_message("success: packet write acknowledgement".to_string());

        let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
        let event = IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
            conn_id_on_b.clone(),
        ));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
        ctx_b.emit_ibc_event(event);
        let event = IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            msg.packet,
            acknowledgement,
            conn_id_on_b.clone(),
        ));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
        ctx_b.emit_ibc_event(event);

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::Module(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let chan_end_path_on_b =
        ChannelEndPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    chan_end_on_b.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_a.clone(),
        Some(msg.packet.chan_id_on_a.clone()),
    );

    chan_end_on_b.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
    let conn_end_on_b = ctx_b.connection_end(conn_id_on_b)?;

    conn_end_on_b.verify_state_matches(&ConnectionState::Open)?;

    let latest_height = ctx_b.host_height()?;
    if msg.packet.timeout_height_on_b.has_expired(latest_height) {
        return Err(PacketError::LowPacketHeight {
            chain_height: latest_height,
            timeout_height: msg.packet.timeout_height_on_b,
        }
        .into());
    }

    let latest_timestamp = ctx_b.host_timestamp()?;
    if let Expiry::Expired = latest_timestamp.check_expiry(&msg.packet.timeout_timestamp_on_b) {
        return Err(PacketError::LowPacketTimestamp.into());
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;

        client_state_of_a_on_b.confirm_not_frozen()?;
        client_state_of_a_on_b.validate_proof_height(msg.proof_height_on_a)?;

        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(client_id_on_b, &msg.proof_height_on_a);
        let consensus_state_of_a_on_b = ctx_b.consensus_state(&client_cons_state_path_on_b)?;

        let expected_commitment_on_a = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );
        let commitment_path_on_a = CommitmentPath::new(
            &msg.packet.port_id_on_a,
            &msg.packet.chan_id_on_a,
            msg.packet.seq_on_a,
        );

        verify_conn_delay_passed(ctx_b, msg.proof_height_on_a, &conn_end_on_b)?;

        // Verify the proof for the packet against the chain store.
        client_state_of_a_on_b
            .verify_membership(
                conn_end_on_b.counterparty().prefix(),
                &msg.proof_commitment_on_a,
                consensus_state_of_a_on_b.root(),
                Path::Commitment(commitment_path_on_a),
                expected_commitment_on_a.into_vec(),
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.packet.seq_on_a,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    if chan_end_on_b.order_matches(&Order::Ordered) {
        let seq_recv_path_on_b =
            SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
        let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
        if msg.packet.seq_on_a > next_seq_recv {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: msg.packet.seq_on_a,
                next_sequence: next_seq_recv,
            }
            .into());
        }

        if msg.packet.seq_on_a == next_seq_recv {
            // Case where the recvPacket is successful and an
            // acknowledgement will be written (not a no-op)
            validate_write_acknowledgement(ctx_b, msg)?;
        }
    } else {
        let receipt_path_on_b = ReceiptPath::new(
            &msg.packet.port_id_on_a,
            &msg.packet.chan_id_on_a,
            msg.packet.seq_on_a,
        );
        let packet_rec = ctx_b.get_packet_receipt(&receipt_path_on_b);
        match packet_rec {
            Ok(_receipt) => {}
            Err(ContextError::PacketError(PacketError::PacketReceiptNotFound { sequence }))
                if sequence == msg.packet.seq_on_a => {}
            Err(e) => return Err(e),
        }
        // Case where the recvPacket is successful and an
        // acknowledgement will be written (not a no-op)
        validate_write_acknowledgement(ctx_b, msg)?;
    };

    Ok(())
}

fn validate_write_acknowledgement<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let packet = msg.packet.clone();
    let ack_path_on_b = AckPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);
    if ctx_b.get_packet_acknowledgement(&ack_path_on_b).is_ok() {
        return Err(PacketError::AcknowledgementExists {
            sequence: msg.packet.seq_on_a,
        }
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::msgs::recv_packet::test_util::get_dummy_raw_msg_recv_packet;
    use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
    use crate::core::ics04_channel::packet::Packet;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::core::router::Router;
    use crate::core::timestamp::Timestamp;
    use crate::core::timestamp::ZERO_DURATION;
    use crate::Height;

    use crate::mock::context::MockContext;
    use crate::mock::ics18_relayer::context::RelayerContext;
    use crate::mock::router::MockRouter;
    use crate::test_utils::get_dummy_account_id;
    use crate::test_utils::DummyTransferModule;

    pub struct Fixture {
        pub context: MockContext,
        pub router: MockRouter,
        pub client_height: Height,
        pub host_height: Height,
        pub msg: MsgRecvPacket,
        pub conn_end_on_b: ConnectionEnd,
        pub chan_end_on_b: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let context = MockContext::default();

        let mut router = MockRouter::default();
        router
            .add_route(PortId::transfer(), DummyTransferModule::new())
            .unwrap();

        let host_height = context.query_latest_height().unwrap().increment();

        let client_height = host_height.increment();

        let msg = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(
            client_height.revision_height(),
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let chan_end_on_b = ChannelEnd::new(
            State::Open,
            Order::default(),
            Counterparty::new(packet.port_id_on_a, Some(packet.chan_id_on_a)),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        )
        .unwrap();

        let conn_end_on_b = ConnectionEnd::new(
            ConnectionState::Open,
            ClientId::default(),
            ConnectionCounterparty::new(
                ClientId::default(),
                Some(ConnectionId::default()),
                Default::default(),
            ),
            get_compatible_versions(),
            ZERO_DURATION,
        )
        .unwrap();

        Fixture {
            context,
            router,
            client_height,
            host_height,
            msg,
            conn_end_on_b,
            chan_end_on_b,
        }
    }

    #[rstest]
    fn recv_packet_fail_no_channel(fixture: Fixture) {
        let Fixture { context, msg, .. } = fixture;

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn recv_packet_validate_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_b,
            chan_end_on_b,
            client_height,
            host_height,
            ..
        } = fixture;

        let packet = &msg.packet;
        let mut context = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_b)
            .with_channel(
                packet.port_id_on_b.clone(),
                packet.chan_id_on_b.clone(),
                chan_end_on_b,
            )
            .with_send_sequence(
                packet.port_id_on_b.clone(),
                packet.chan_id_on_b.clone(),
                1.into(),
            )
            .with_height(host_height)
            // This `with_recv_sequence` is required for ordered channels
            .with_recv_sequence(
                packet.port_id_on_b.clone(),
                packet.chan_id_on_b.clone(),
                packet.seq_on_a,
            );

        context
            .store_update_time(
                ClientId::default(),
                client_height,
                Timestamp::from_nanoseconds(1000).unwrap(),
            )
            .unwrap();
        context
            .store_update_height(
                ClientId::default(),
                client_height,
                Height::new(0, 5).unwrap(),
            )
            .unwrap();

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Happy path: validation should succeed. err: {res:?}"
        )
    }

    #[rstest]
    fn recv_packet_timeout_expired(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_b,
            chan_end_on_b,
            client_height,
            host_height,
            ..
        } = fixture;

        let packet_old = Packet {
            seq_on_a: 1.into(),
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            port_id_on_b: PortId::transfer(),
            chan_id_on_b: ChannelId::default(),
            data: Vec::new(),
            timeout_height_on_b: client_height.into(),
            timeout_timestamp_on_b: Timestamp::from_nanoseconds(1).unwrap(),
        };

        let msg_packet_old = MsgRecvPacket::new(
            packet_old,
            msg.proof_commitment_on_a.clone(),
            msg.proof_height_on_a,
            get_dummy_account_id(),
        );
        let context = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_b)
            .with_channel(PortId::transfer(), ChannelId::default(), chan_end_on_b)
            .with_send_sequence(PortId::transfer(), ChannelId::default(), 1.into())
            .with_height(host_height);

        let res = validate(&context, &msg_packet_old);

        assert!(
            res.is_err(),
            "recv_packet validation should fail when the packet has timed out"
        )
    }

    #[rstest]
    fn recv_packet_execute_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            mut router,
            msg,
            conn_end_on_b,
            chan_end_on_b,
            client_height,
            ..
        } = fixture;
        let mut ctx = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_b)
            .with_channel(PortId::transfer(), ChannelId::default(), chan_end_on_b);

        let module = router.get_route_mut(&msg.packet.port_id_on_b).unwrap();
        let res = recv_packet_execute(&mut ctx, module, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 4);
        assert!(matches!(
            &ctx.events[0],
            &IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(&ctx.events[1], &IbcEvent::ReceivePacket(_)));
        assert!(matches!(
            &ctx.events[2],
            &IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(&ctx.events[3], &IbcEvent::WriteAcknowledgement(_)));
    }
}
