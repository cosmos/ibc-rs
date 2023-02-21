use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, SeqAckPath,
};
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgAcknowledgement) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let packet = &msg.packet;
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_on_a, &packet.chan_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    if !chan_end_on_a.state_matches(&State::Open) {
        return Err(PacketError::ChannelClosed {
            channel_id: packet.chan_on_a.clone(),
        }
        .into());
    }

    let counterparty = Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b.clone()));

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.port_on_b.clone(),
            channel_id: packet.chan_on_b.clone(),
        }
        .into());
    }

    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];
    let conn_end_on_a = ctx_a.connection_end(conn_id_on_a)?;

    if !conn_end_on_a.state_matches(&ConnectionState::Open) {
        return Err(PacketError::ConnectionNotOpen {
            connection_id: chan_end_on_a.connection_hops()[0].clone(),
        }
        .into());
    }

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence);

    // Verify packet commitment
    let commitment_on_a = match ctx_a.get_packet_commitment(&commitment_path_on_a) {
        Ok(commitment_on_a) => commitment_on_a,

        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        Err(_) => return Ok(()),
    };

    if commitment_on_a
        != ctx_a.packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        )
    {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.sequence,
        }
        .into());
    }

    if let Order::Ordered = chan_end_on_a.ordering {
        let seq_ack_path_on_a = SeqAckPath::new(&packet.port_on_a, &packet.chan_on_a);
        let next_seq_ack = ctx_a.get_next_sequence_ack(&seq_ack_path_on_a)?;
        if packet.sequence != next_seq_ack {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: packet.sequence,
                next_sequence: next_seq_ack,
            }
            .into());
        }
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_on_a = ctx_a.client_state(client_id_on_a)?;

        // The client must not be frozen.
        if client_state_on_a.is_frozen() {
            return Err(PacketError::FrozenClient {
                client_id: client_id_on_a.clone(),
            }
            .into());
        }
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let ack_commitment = ctx_a.ack_commitment(&msg.acknowledgement);
        let ack_path_on_b = AckPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence);
        // Verify the proof for the packet against the chain store.
        client_state_on_a
            .verify_packet_acknowledgement(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_acked_on_b,
                consensus_state.root(),
                &ack_path_on_b,
                ack_commitment,
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: packet.sequence,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::handler::acknowledgement::validate;
    use crate::core::ics24_host::identifier::ChannelId;
    use crate::core::ics24_host::identifier::PortId;
    use crate::core::ValidationContext;
    use crate::prelude::*;
    use rstest::*;
    use test_log::test;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_acknowledgement;
    use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;

    pub struct Fixture {
        pub context: MockContext,
        pub client_height: Height,
        pub msg: MsgAcknowledgement,
        pub packet_commitment: PacketCommitment,
        pub conn_end_on_a: ConnectionEnd,
        pub chan_end_on_a: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let context = MockContext::default();

        let client_height = Height::new(0, 2).unwrap();

        let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
            client_height.revision_height(),
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let packet_commitment = context.packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a = ChannelEnd::new(
            State::Open,
            Order::default(),
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

        Fixture {
            context,
            client_height,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a,
        }
    }

    #[rstest]
    fn ack_fail_no_channel(fixture: Fixture) {
        let Fixture { context, msg, .. } = fixture;

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    /// NO-OP case
    #[rstest]
    fn ack_success_no_packet_commitment(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_a,
            chan_end_on_a,
            client_height,
            ..
        } = fixture;
        let context = context
            .with_client(&ClientId::default(), client_height)
            .with_channel(PortId::default(), ChannelId::default(), chan_end_on_a)
            .with_connection(ConnectionId::default(), conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Validation should succeed when no packet commitment is present"
        )
    }

    #[rstest]
    fn ack_success_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a,
            client_height,
            ..
        } = fixture;
        let context = context
            .with_client(&ClientId::default(), client_height)
            .with_channel(PortId::default(), ChannelId::default(), chan_end_on_a)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_packet_commitment(
                msg.packet.port_on_a.clone(),
                msg.packet.chan_on_a.clone(),
                msg.packet.sequence,
                packet_commitment,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Happy path: validation should succeed. err: {res:?}"
        )
    }
}
