use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
};
use crate::prelude::*;

use crate::core::{ContextError, ValidationContext};

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeoutOnClose) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let packet = &msg.packet;
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_on_a, &packet.chan_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    let counterparty = Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b.clone()));

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.port_on_b.clone(),
            channel_id: packet.chan_on_b.clone(),
        }
        .into());
    }

    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_on_a,
        &msg.packet.chan_on_a,
        msg.packet.sequence,
    );

    //verify the packet was sent, check the store
    let commitment_on_a = match ctx_a.get_packet_commitment(&commitment_path_on_a) {
        Ok(commitment_on_a) => commitment_on_a,

        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        Err(_) => return Ok(()),
    };

    let expected_commitment_on_a = ctx_a.packet_commitment(
        &packet.data,
        &packet.timeout_height_on_b,
        &packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.sequence,
        }
        .into());
    }

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a.connection_end(&conn_id_on_a)?;

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;

        // The client must not be frozen.
        if client_state_of_b_on_a.is_frozen() {
            return Err(PacketError::FrozenClient {
                client_id: client_id_on_a.clone(),
            }
            .into());
        }
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
        let chan_id_on_b =
            chan_end_on_a
                .counterparty()
                .channel_id()
                .ok_or(PacketError::Channel(
                    ChannelError::InvalidCounterpartyChannelId,
                ))?;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            PacketError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;
        let expected_conn_hops_on_b = vec![conn_id_on_b.clone()];
        let expected_counterparty =
            Counterparty::new(packet.port_on_a.clone(), Some(packet.chan_on_a.clone()));
        let expected_chan_end_on_b = ChannelEnd::new(
            State::Closed,
            *chan_end_on_a.ordering(),
            expected_counterparty,
            expected_conn_hops_on_b,
            chan_end_on_a.version().clone(),
        );

        let chan_end_path_on_b = ChannelEndPath(port_id_on_b, chan_id_on_b.clone());

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_channel_state(
                msg.proof_height_on_b,
                prefix_on_b,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &chan_end_path_on_b,
                &expected_chan_end_on_b,
            )
            .map_err(ChannelError::VerifyChannelFailed)
            .map_err(PacketError::Channel)?;

        let next_seq_recv_verification_result = if chan_end_on_a.order_matches(&Order::Ordered) {
            if packet.sequence < msg.next_seq_recv_on_b {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: packet.sequence,
                    next_sequence: msg.next_seq_recv_on_b,
                }
                .into());
            }
            let seq_recv_path_on_b = SeqRecvPath::new(&packet.port_on_b, &packet.chan_on_b);
            client_state_of_b_on_a.verify_next_sequence_recv(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &seq_recv_path_on_b,
                packet.sequence,
            )
        } else {
            let receipt_path_on_b = ReceiptPath::new(
                &msg.packet.port_on_b,
                &msg.packet.chan_on_b,
                msg.packet.sequence,
            );
            client_state_of_b_on_a.verify_packet_receipt_absence(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &receipt_path_on_b,
            )
        };
        next_seq_recv_verification_result
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_seq_recv_on_b,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics04_channel::handler::timeout_on_close::validate;
    use crate::core::ValidationContext;
    use crate::mock::context::MockContext;
    use crate::prelude::*;
    use crate::Height;
    use rstest::*;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close;
    use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::timestamp::ZERO_DURATION;

    pub struct Fixture {
        pub context: MockContext,
        pub msg: MsgTimeoutOnClose,
        pub packet_commitment: PacketCommitment,
        pub conn_end_on_a: ConnectionEnd,
        pub chan_end_on_a: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let client_height = Height::new(0, 2).unwrap();
        let context = MockContext::default().with_client(&ClientId::default(), client_height);

        let height = 2;
        let timeout_timestamp = 5;

        let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
            height,
            timeout_timestamp,
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let packet_commitment = context.packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a = ChannelEnd::new(
            State::Open,
            Order::Ordered,
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
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a,
        }
    }

    #[rstest]
    fn timeout_on_close_fail_no_channel(fixture: Fixture) {
        let Fixture { context, msg, .. } = fixture;

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    /// NO-OP case
    #[rstest]
    fn timeout_on_close_success_no_packet_commitment(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            conn_end_on_a,
            chan_end_on_a,
            ..
        } = fixture;
        let context = context
            .with_channel(PortId::default(), ChannelId::default(), chan_end_on_a)
            .with_connection(ConnectionId::default(), conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(
            res.is_ok(),
            "Validation should succeed when no packet commitment is present"
        )
    }

    #[rstest]
    fn timeout_on_close_success_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a,
            ..
        } = fixture;
        let context = context
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
