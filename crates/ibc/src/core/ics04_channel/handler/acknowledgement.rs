use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics02_client::client_state::{ClientStateCommon, ClientStateValidation};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics03_connection::delay::verify_conn_delay_passed;
use crate::core::ics04_channel::channel::{Counterparty, Order, State as ChannelState};
use crate::core::ics04_channel::commitment::{compute_ack_commitment, compute_packet_commitment};
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::events::AcknowledgePacket;
use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, SeqAckPath,
};
use crate::core::router::Module;
use crate::core::{ContextError, ExecutionContext, ValidationContext};
use crate::prelude::*;

pub(crate) fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    module
        .on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
        .map_err(ContextError::PacketError)
}

pub(crate) fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
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
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
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
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgAcknowledgement) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    chan_end_on_a.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(
        packet.port_id_on_b.clone(),
        Some(packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];
    let conn_end_on_a = ctx_a.connection_end(conn_id_on_a)?;

    conn_end_on_a.verify_state_matches(&ConnectionState::Open)?;

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a);

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
        != compute_packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        )
    {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.seq_on_a,
        }
        .into());
    }

    if let Order::Ordered = chan_end_on_a.ordering {
        let seq_ack_path_on_a = SeqAckPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
        let next_seq_ack = ctx_a.get_next_sequence_ack(&seq_ack_path_on_a)?;
        if packet.seq_on_a != next_seq_ack {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: packet.seq_on_a,
                next_sequence: next_seq_ack,
            }
            .into());
        }
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;

        {
            let status = client_state_of_b_on_a
                .status(ctx_a.get_client_validation_context(), client_id_on_a)?;
            if !status.is_active() {
                return Err(ClientError::ClientNotActive { status }.into());
            }
        }
        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let ack_commitment = compute_ack_commitment(&msg.acknowledgement);
        let ack_path_on_b =
            AckPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        // Verify the proof for the packet against the chain store.
        client_state_of_b_on_a
            .verify_membership(
                conn_end_on_a.counterparty().prefix(),
                &msg.proof_acked_on_b,
                consensus_state_of_b_on_a.root(),
                Path::Ack(ack_path_on_b),
                ack_commitment.into_vec(),
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: packet.seq_on_a,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use test_log::test;

    use super::*;
    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::{
        ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
    };
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_acknowledgement;
    use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::core::router::{ModuleId, Router};
    use crate::core::timestamp::{Timestamp, ZERO_DURATION};
    use crate::mock::context::MockContext;
    use crate::mock::router::MockRouter;
    use crate::test_utils::DummyTransferModule;

    struct Fixture {
        ctx: MockContext,
        router: MockRouter,
        client_height: Height,
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
        let ctx = MockContext::default().with_client(&ClientId::default(), client_height);
        let mut router = MockRouter::default();

        let module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());
        let module = DummyTransferModule::new();
        router.add_route(module_id.clone(), module).unwrap();

        let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
            client_height.revision_height(),
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let packet_commitment = compute_packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a_unordered = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(packet.port_id_on_b, Some(packet.chan_id_on_b)),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        )
        .unwrap();

        let mut chan_end_on_a_ordered = chan_end_on_a_unordered.clone();
        chan_end_on_a_ordered.ordering = Order::Ordered;

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
        )
        .unwrap();

        Fixture {
            ctx,
            router,
            client_height,
            module_id,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_unordered,
            chan_end_on_a_ordered,
        }
    }

    #[rstest]
    fn ack_fail_no_channel(fixture: Fixture) {
        let Fixture { ctx, msg, .. } = fixture;

        let res = validate(&ctx, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    /// NO-OP case
    #[rstest]
    fn ack_success_no_packet_commitment(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            conn_end_on_a,
            chan_end_on_a_unordered,
            client_height,
            ..
        } = fixture;
        let ctx = ctx
            .with_client(&ClientId::default(), client_height)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a);

        let res = validate(&ctx, &msg);

        assert!(
            res.is_ok(),
            "Validation should succeed when no packet commitment is present"
        )
    }

    #[rstest]
    fn ack_success_happy_path(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_unordered,
            client_height,
            ..
        } = fixture;
        let mut ctx: MockContext = ctx
            .with_client(&ClientId::default(), client_height)
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
        ctx.store_update_time(
            ClientId::default(),
            client_height,
            Timestamp::from_nanoseconds(1000).unwrap(),
        )
        .unwrap();
        ctx.store_update_height(
            ClientId::default(),
            client_height,
            Height::new(0, 4).unwrap(),
        )
        .unwrap();

        let res = validate(&ctx, &msg);

        assert!(
            res.is_ok(),
            "Happy path: validation should succeed. err: {res:?}"
        )
    }

    #[rstest]
    fn ack_unordered_chan_execute(fixture: Fixture) {
        let Fixture {
            ctx,
            mut router,
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

        let module = router.get_route_mut(&module_id).unwrap();
        let res = acknowledgement_packet_execute(&mut ctx, module, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
    }

    #[rstest]
    fn ack_ordered_chan_execute(fixture: Fixture) {
        let Fixture {
            ctx,
            mut router,
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

        let module = router.get_route_mut(&module_id).unwrap();
        let res = acknowledgement_packet_execute(&mut ctx, module, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
    }
}
