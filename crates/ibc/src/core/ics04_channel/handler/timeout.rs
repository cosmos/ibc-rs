use crate::core::ics02_client::client_state::StaticClientStateBase;
use crate::core::ics02_client::consensus_state::StaticConsensusState;
use crate::prelude::*;
use prost::Message;

use crate::core::events::IbcEvent;
use crate::core::events::MessageEvent;
use crate::core::ics03_connection::delay::verify_conn_delay_passed;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::commitment::compute_packet_commitment;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::events::ChannelClosed;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics04_channel::{events::TimeoutPacket, handler::timeout_on_close};
use crate::core::ics24_host::path::Path;
use crate::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
};
use crate::core::router::ModuleId;
use crate::core::{ContextError, ExecutionContext, ValidationContext};

pub(crate) enum TimeoutMsgType {
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}

pub(crate) fn timeout_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    match &timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => validate(ctx_a, msg),
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

pub(crate) fn timeout_packet_execute<ExecCtx>(
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
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
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
            ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
            ctx_a.emit_ibc_event(event);
        }

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeout) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let chan_end_on_a = ctx_a.channel_end(&ChannelEndPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
    ))?;

    chan_end_on_a.verify_state_matches(&State::Open)?;

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_b.clone(),
        Some(msg.packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a.connection_end(&conn_id_on_a)?;

    //verify packet commitment
    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
    );
    let commitment_on_a = match ctx_a.get_packet_commitment(&commitment_path_on_a) {
        Ok(commitment_on_a) => commitment_on_a,

        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        Err(_) => return Ok(()),
    };

    let expected_commitment_on_a = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: msg.packet.seq_on_a,
        }
        .into());
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
        client_state_of_b_on_a.confirm_not_frozen()?;
        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        // check that timeout height or timeout timestamp has passed on the other end
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let timestamp_of_b = consensus_state_of_b_on_a.timestamp();

        if !msg.packet.timed_out(&timestamp_of_b, msg.proof_height_on_b) {
            return Err(PacketError::PacketTimeoutNotReached {
                timeout_height: msg.packet.timeout_height_on_b,
                chain_height: msg.proof_height_on_b,
                timeout_timestamp: msg.packet.timeout_timestamp_on_b,
                chain_timestamp: timestamp_of_b,
            }
            .into());
        }

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        let next_seq_recv_verification_result = if chan_end_on_a.order_matches(&Order::Ordered) {
            if msg.packet.seq_on_a < msg.next_seq_recv_on_b {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: msg.packet.seq_on_a,
                    next_sequence: msg.next_seq_recv_on_b,
                }
                .into());
            }
            let seq_recv_path_on_b =
                SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);

            let mut value = Vec::new();
            u64::from(msg.packet.seq_on_a)
                .encode(&mut value)
                .map_err(|_| PacketError::CannotEncodeSequence {
                    sequence: msg.packet.seq_on_a,
                })?;

            client_state_of_b_on_a.verify_membership(
                conn_end_on_a.counterparty().prefix(),
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                Path::SeqRecv(seq_recv_path_on_b),
                value,
            )
        } else {
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
        };
        next_seq_recv_verification_result
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_seq_recv_on_b,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::commitment::PacketCommitment;
    use crate::core::ics04_channel::handler::timeout::validate;
    use crate::core::ics04_channel::msgs::timeout::test_util::get_dummy_raw_msg_timeout;
    use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::core::timestamp::Timestamp;
    use crate::core::timestamp::ZERO_DURATION;

    use crate::applications::transfer::MODULE_ID_STR;
    use crate::mock::context::MockContext;
    use crate::test_utils::DummyTransferModule;

    struct Fixture {
        ctx: MockContext,
        client_height: Height,
        module_id: ModuleId,
        msg: MsgTimeout,
        packet_commitment: PacketCommitment,
        conn_end_on_a: ConnectionEnd,
        chan_end_on_a_ordered: ChannelEnd,
        chan_end_on_a_unordered: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let client_height = Height::new(0, 2).unwrap();
        let mut ctx = MockContext::default().with_client(&ClientId::default(), client_height);

        let client_height = Height::new(0, 2).unwrap();

        let module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());
        let module = DummyTransferModule::new();
        ctx.add_route(module_id.clone(), module).unwrap();

        let msg_proof_height = 2;
        let msg_timeout_height = 5;
        let timeout_timestamp = Timestamp::now().nanoseconds();

        let msg = MsgTimeout::try_from(get_dummy_raw_msg_timeout(
            msg_proof_height,
            msg_timeout_height,
            timeout_timestamp,
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
            Counterparty::new(packet.port_id_on_b.clone(), Some(packet.chan_id_on_b)),
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
            client_height,
            module_id,
            msg,
            packet_commitment,
            conn_end_on_a,
            chan_end_on_a_ordered,
            chan_end_on_a_unordered,
        }
    }

    #[rstest]
    fn timeout_fail_no_channel(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            client_height,
            ..
        } = fixture;
        let ctx = ctx.with_client(&ClientId::default(), client_height);
        let res = validate(&ctx, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn timeout_fail_no_consensus_state_for_height(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            packet_commitment,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let ctx = ctx
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        let res = validate(&ctx, &msg);

        assert!(
            res.is_err(),
            "Validation fails because the client does not have a consensus state for the required height"
        )
    }

    #[rstest]
    fn timeout_fail_proof_timeout_not_reached(fixture: Fixture) {
        let Fixture {
            ctx,
            mut msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            client_height,
            ..
        } = fixture;

        // timeout timestamp has not reached yet
        let timeout_timestamp_on_b =
            (msg.packet.timeout_timestamp_on_b + core::time::Duration::new(10, 0)).unwrap();
        msg.packet.timeout_timestamp_on_b = timeout_timestamp_on_b;
        let packet_commitment = compute_packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let packet = msg.packet.clone();

        let mut ctx = ctx
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
                packet_commitment,
            );

        ctx.store_update_time(
            ClientId::default(),
            client_height,
            Timestamp::from_nanoseconds(5).unwrap(),
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
            res.is_err(),
            "Validation should fail because the timeout height was reached, but the timestamp hasn't been reached. Both the height and timestamp need to be reached for the packet to be considered timed out"
        )
    }

    /// NO-OP case
    #[rstest]
    fn timeout_success_no_packet_commitment(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            conn_end_on_a,
            chan_end_on_a_unordered,
            ..
        } = fixture;
        let ctx = ctx
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
    fn timeout_unordered_channel_validate(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            chan_end_on_a_unordered,
            conn_end_on_a,
            packet_commitment,
            client_height,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let mut ctx = ctx
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_unordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
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
            Height::new(0, 5).unwrap(),
        )
        .unwrap();

        let res = validate(&ctx, &msg);

        assert!(res.is_ok(), "Good parameters for unordered channels")
    }

    #[rstest]
    fn timeout_ordered_channel_validate(fixture: Fixture) {
        let Fixture {
            ctx,
            msg,
            chan_end_on_a_ordered,
            conn_end_on_a,
            packet_commitment,
            client_height,
            ..
        } = fixture;

        let packet = msg.packet.clone();

        let mut ctx = ctx
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a)
            .with_channel(
                PortId::default(),
                ChannelId::default(),
                chan_end_on_a_ordered,
            )
            .with_packet_commitment(
                packet.port_id_on_a,
                packet.chan_id_on_a,
                packet.seq_on_a,
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

        assert!(res.is_ok(), "Good parameters for unordered channels")
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

        let res = timeout_packet_execute(&mut ctx, module_id, TimeoutMsgType::Timeout(msg));

        assert!(res.is_ok());

        // Unordered channels only emit one event
        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
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

        let res = timeout_packet_execute(&mut ctx, module_id, TimeoutMsgType::Timeout(msg));

        assert!(res.is_ok());

        // Ordered channels emit 2 events
        assert_eq!(ctx.events.len(), 4);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::TimeoutPacket(_)));
        assert!(matches!(
            ctx.events[2],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[3], IbcEvent::ChannelClosed(_)));
    }
}
