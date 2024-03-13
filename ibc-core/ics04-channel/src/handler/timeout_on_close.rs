use ibc_core_channel_types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc_core_channel_types::commitment::compute_packet_commitment;
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_channel_types::msgs::MsgTimeoutOnClose;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::delay::verify_conn_delay_passed;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, ReceiptPath, SeqRecvPath,
};
use ibc_core_host::ValidationContext;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Protobuf;

pub fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeoutOnClose) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    let counterparty = Counterparty::new(
        packet.port_id_on_b.clone(),
        Some(packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
    );

    //verify the packet was sent, check the store
    let Ok(commitment_on_a) = ctx_a.get_packet_commitment(&commitment_path_on_a) else {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let expected_commitment_on_a = compute_packet_commitment(
        &packet.data,
        &packet.timeout_height_on_b,
        &packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.seq_on_a,
        }
        .into());
    }

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a.connection_end(&conn_id_on_a)?;

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_val_ctx_a = ctx_a.get_client_validation_context();
        let client_state_of_b_on_a = client_val_ctx_a.client_state(client_id_on_a)?;

        client_state_of_b_on_a
            .status(ctx_a.get_client_validation_context(), client_id_on_a)?
            .verify_is_active()?;

        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            client_id_on_a.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
        let chan_id_on_b = chan_end_on_a
            .counterparty()
            .channel_id()
            .ok_or(PacketError::Channel(ChannelError::MissingCounterparty))?;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            PacketError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;
        let expected_conn_hops_on_b = vec![conn_id_on_b.clone()];
        let expected_counterparty = Counterparty::new(
            packet.port_id_on_a.clone(),
            Some(packet.chan_id_on_a.clone()),
        );
        let expected_chan_end_on_b = ChannelEnd::new(
            State::Closed,
            *chan_end_on_a.ordering(),
            expected_counterparty,
            expected_conn_hops_on_b,
            chan_end_on_a.version().clone(),
        )?;

        let chan_end_path_on_b = ChannelEndPath(port_id_on_b, chan_id_on_b.clone());

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_membership(
                prefix_on_b,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                Path::ChannelEnd(chan_end_path_on_b),
                expected_chan_end_on_b.encode_vec(),
            )
            .map_err(ChannelError::VerifyChannelFailed)
            .map_err(PacketError::Channel)?;

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        let next_seq_recv_verification_result = match chan_end_on_a.ordering {
            Order::Ordered => {
                if packet.seq_on_a < msg.next_seq_recv_on_b {
                    return Err(PacketError::InvalidPacketSequence {
                        given_sequence: packet.seq_on_a,
                        next_sequence: msg.next_seq_recv_on_b,
                    }
                    .into());
                }
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&packet.port_id_on_b, &packet.chan_id_on_b);

                client_state_of_b_on_a.verify_membership(
                    conn_end_on_a.counterparty().prefix(),
                    &msg.proof_unreceived_on_b,
                    consensus_state_of_b_on_a.root(),
                    Path::SeqRecv(seq_recv_path_on_b),
                    packet.seq_on_a.to_vec(),
                )
            }
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath::new(
                    &msg.packet.port_id_on_b,
                    &msg.packet.chan_id_on_b,
                    msg.packet.seq_on_a,
                );

                client_state_of_b_on_a.verify_non_membership(
                    conn_end_on_a.counterparty().prefix(),
                    &msg.proof_unreceived_on_b,
                    consensus_state_of_b_on_a.root(),
                    Path::Receipt(receipt_path_on_b),
                )
            }
            Order::None => {
                return Err(ContextError::ChannelError(ChannelError::InvalidOrderType {
                    expected: "Channel ordering cannot be None".to_string(),
                    actual: chan_end_on_a.ordering.to_string(),
                }))
            }
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
