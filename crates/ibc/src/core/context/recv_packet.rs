use crate::{
    core::{
        ics04_channel::{
            channel::Order,
            error::ChannelError,
            events::{ReceivePacket, WriteAcknowledgement},
            handler::recv_packet,
            msgs::recv_packet::MsgRecvPacket,
            packet::Receipt,
        },
        ics24_host::path::{AckPath, ChannelEndPath, ReceiptPath, SeqRecvPath},
        ics26_routing::context::ModuleId,
    },
    events::IbcEvent,
    prelude::*,
};

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn recv_packet_validate<ValCtx>(
    ctx_b: &ValCtx,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    // Note: this contains the validation for `write_acknowledgement` as well.
    recv_packet::validate(ctx_b, &msg)

    // nothing to validate with the module, since `onRecvPacket` cannot fail.
}

pub(super) fn recv_packet_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_b = ChannelEndPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let packet = msg.packet.clone();
                let receipt_path_on_b =
                    ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence);
                ctx_b.get_packet_receipt(&receipt_path_on_b).is_ok()
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                msg.packet.sequence < next_seq_recv
            }
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        match chan_end_on_b.ordering {
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath {
                    port_id: msg.packet.port_on_b.clone(),
                    channel_id: msg.packet.chan_on_b.clone(),
                    sequence: msg.packet.sequence,
                };

                ctx_b.store_packet_receipt(&receipt_path_on_b, Receipt::Ok)?;
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
                ctx_b.store_next_sequence_recv(&seq_recv_path_on_b, next_seq_recv.increment())?;
            }
            _ => {}
        }
        let ack_path_on_b = AckPath::new(
            &msg.packet.port_on_b,
            &msg.packet.chan_on_b,
            msg.packet.sequence,
        );
        // `writeAcknowledgement` handler state changes
        ctx_b
            .store_packet_acknowledgement(&ack_path_on_b, ctx_b.ack_commitment(&acknowledgement))?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: packet receive".to_string());
        ctx_b.log_message("success: packet write acknowledgement".to_string());

        let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
        ctx_b.emit_ibc_event(IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
            conn_id_on_b.clone(),
        )));
        ctx_b.emit_ibc_event(IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            msg.packet,
            acknowledgement,
            conn_id_on_b.clone(),
        )));

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        applications::transfer::MODULE_ID_STR,
        core::{
            context::recv_packet::recv_packet_execute,
            ics03_connection::version::get_compatible_versions,
            ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
            ics26_routing::context::ModuleId,
        },
        events::IbcEvent,
        prelude::*,
        test_utils::DummyTransferModule,
        timestamp::ZERO_DURATION,
    };
    use rstest::*;

    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::{
        core::{
            ics03_connection::connection::ConnectionEnd,
            ics04_channel::{
                channel::{ChannelEnd, Counterparty, Order, State},
                msgs::recv_packet::{test_util::get_dummy_raw_msg_recv_packet, MsgRecvPacket},
                Version,
            },
        },
        mock::{context::MockContext, ics18_relayer::context::RelayerContext},
        Height,
    };

    pub struct Fixture {
        pub context: MockContext,
        pub module_id: ModuleId,
        pub client_height: Height,
        pub host_height: Height,
        pub msg: MsgRecvPacket,
        pub conn_end_on_b: ConnectionEnd,
        pub chan_end_on_b: ChannelEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let mut context = MockContext::default();

        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        let module = DummyTransferModule::new(context.ibc_store_share());
        context.add_route(module_id.clone(), module).unwrap();

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
            Counterparty::new(packet.port_on_a, Some(packet.chan_on_a)),
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
            module_id,
            client_height,
            host_height,
            msg,
            conn_end_on_b,
            chan_end_on_b,
        }
    }

    #[rstest]
    fn recv_packet_execute_test(fixture: Fixture) {
        let Fixture {
            context,
            module_id,
            msg,
            conn_end_on_b,
            chan_end_on_b,
            client_height,
            ..
        } = fixture;
        let mut ctx = context
            .with_client(&ClientId::default(), client_height)
            .with_connection(ConnectionId::default(), conn_end_on_b)
            .with_channel(PortId::default(), ChannelId::default(), chan_end_on_b);

        let res = recv_packet_execute(&mut ctx, module_id, msg);

        assert!(res.is_ok());

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(&ctx.events[0], &IbcEvent::ReceivePacket(_)));
        assert!(matches!(&ctx.events[1], &IbcEvent::WriteAcknowledgement(_)));
    }
}
