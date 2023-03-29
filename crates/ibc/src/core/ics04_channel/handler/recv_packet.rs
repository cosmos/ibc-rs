use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics03_connection::delay::verify_conn_delay_passed;
use crate::core::ics04_channel::channel::{Counterparty, Order, State};
use crate::core::ics04_channel::commitment::compute_packet_commitment;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
};
use crate::core::ics24_host::Path;
use crate::timestamp::Expiry;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let chan_end_path_on_b =
        ChannelEndPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    if !chan_end_on_b.state_matches(&State::Open) {
        return Err(PacketError::InvalidChannelState {
            channel_id: msg.packet.chan_id_on_a.clone(),
            state: chan_end_on_b.state,
        }
        .into());
    }

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_a.clone(),
        Some(msg.packet.chan_id_on_a.clone()),
    );

    if !chan_end_on_b.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: msg.packet.port_id_on_a.clone(),
            channel_id: msg.packet.chan_id_on_a.clone(),
        }
        .into());
    }

    let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
    let conn_end_on_b = ctx_b.connection_end(conn_id_on_b)?;

    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(PacketError::ConnectionNotOpen {
            connection_id: chan_end_on_b.connection_hops()[0].clone(),
        }
        .into());
    }

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
    use crate::core::ics04_channel::handler::recv_packet::validate;
    use crate::core::ExecutionContext;
    use crate::prelude::*;
    use crate::Height;
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
    use crate::mock::context::MockContext;
    use crate::mock::ics18_relayer::context::RelayerContext;
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::timestamp::ZERO_DURATION;

    pub struct Fixture {
        pub context: MockContext,
        pub client_height: Height,
        pub host_height: Height,
        pub msg: MsgRecvPacket,
        pub conn_end_on_b: ConnectionEnd,
        pub chan_end_on_b: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let context = MockContext::default();

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
        );

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
        );

        Fixture {
            context,
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
    fn recv_packet_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_b,
            chan_end_on_b,
            client_height,
            host_height,
            ..
        } = fixture;

        let packet = msg.packet.clone();
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
            port_id_on_a: PortId::default(),
            chan_id_on_a: ChannelId::default(),
            port_id_on_b: PortId::default(),
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
            .with_channel(PortId::default(), ChannelId::default(), chan_end_on_b)
            .with_send_sequence(PortId::default(), ChannelId::default(), 1.into())
            .with_height(host_height);

        let res = validate(&context, &msg_packet_old);

        assert!(
            res.is_err(),
            "recv_packet validation should fail when the packet has timed out"
        )
    }
}
