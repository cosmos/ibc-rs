use core::str::FromStr;

use ibc::apps::transfer::handler::send_transfer;
use ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
use ibc::apps::transfer::types::packet::PacketData;
use ibc::apps::transfer::types::PrefixedCoin;
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::timeout::TimeoutHeight;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc::primitives::Timestamp;

use crate::context::MockContext;
use crate::fixtures::core::signer::dummy_account_id;
use crate::hosts::{HostClientState, TestHost};
use crate::relayer::context::RelayerContext;
use crate::testapp::ibc::applications::transfer::types::DummyTransferModule;
use crate::testapp::ibc::core::types::DefaultIbcStore;

pub fn ibc_integration_test<A, B>()
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    let ctx_a = MockContext::<A>::default();
    let ctx_b = MockContext::<B>::default();

    let signer = dummy_account_id();

    let mut relayer = RelayerContext::new(ctx_a, ctx_b);

    // client creation
    let client_id_on_a = relayer.create_client_on_a(signer.clone());
    let client_id_on_b = relayer.create_client_on_b(signer.clone());

    // connection from A to B
    let (conn_id_on_a, conn_id_on_b) = relayer.create_connection_on_a(
        client_id_on_a.clone(),
        client_id_on_b.clone(),
        signer.clone(),
    );

    assert_eq!(conn_id_on_a, ConnectionId::new(0));
    assert_eq!(conn_id_on_b, ConnectionId::new(0));

    // connection from B to A
    let (conn_id_on_b, conn_id_on_a) = relayer.create_connection_on_b(
        client_id_on_b.clone(),
        client_id_on_a.clone(),
        signer.clone(),
    );

    assert_eq!(conn_id_on_a, ConnectionId::new(1));
    assert_eq!(conn_id_on_b, ConnectionId::new(1));

    // channel from A to B
    let (chan_id_on_a, chan_id_on_b) = relayer.create_channel_on_a(
        conn_id_on_a.clone(),
        PortId::transfer(),
        conn_id_on_b.clone(),
        PortId::transfer(),
        signer.clone(),
    );

    assert_eq!(chan_id_on_a, ChannelId::new(0));
    assert_eq!(chan_id_on_b, ChannelId::new(0));

    // close the channel from A to B
    relayer.close_channel_on_a(
        chan_id_on_a.clone(),
        PortId::transfer(),
        chan_id_on_b.clone(),
        PortId::transfer(),
        signer.clone(),
    );

    // channel from B to A
    let (chan_id_on_b, chan_id_on_a) = relayer.create_channel_on_b(
        conn_id_on_b,
        PortId::transfer(),
        conn_id_on_a,
        PortId::transfer(),
        signer.clone(),
    );

    assert_eq!(chan_id_on_a, ChannelId::new(1));
    assert_eq!(chan_id_on_b, ChannelId::new(1));

    // send packet from A to B

    // generate packet for DummyTransferModule
    let packet_data = PacketData {
        token: PrefixedCoin::from_str("1000uibc").expect("valid prefixed coin"),
        sender: signer.clone(),
        receiver: signer.clone(),
        memo: "sample memo".into(),
    };

    // packet with ibc metadata
    let msg = MsgTransfer {
        port_id_on_a: PortId::transfer(),
        chan_id_on_a: chan_id_on_a.clone(),
        packet_data,
        timeout_height_on_b: TimeoutHeight::Never,
        timeout_timestamp_on_b: Timestamp::none(),
    };

    // module creates the send_packet
    send_transfer(
        relayer.get_ctx_a_mut().ibc_store_mut(),
        &mut DummyTransferModule,
        msg,
    )
    .expect("successfully created send_packet");

    // send_packet wasn't committed, hence produce a block
    relayer.get_ctx_a_mut().advance_block();

    // retrieve the send_packet event
    let Some(IbcEvent::SendPacket(send_packet_event)) = relayer
        .get_ctx_a()
        .ibc_store()
        .events
        .lock()
        .iter()
        .rev()
        .nth(2)
        .cloned()
    else {
        panic!("unexpected event")
    };

    // create the IBC packet type
    let packet = Packet {
        port_id_on_a: send_packet_event.port_id_on_a().clone(),
        chan_id_on_a: send_packet_event.chan_id_on_a().clone(),
        seq_on_a: *send_packet_event.seq_on_a(),
        data: send_packet_event.packet_data().to_vec(),
        timeout_height_on_b: *send_packet_event.timeout_height_on_b(),
        timeout_timestamp_on_b: *send_packet_event.timeout_timestamp_on_b(),
        port_id_on_b: send_packet_event.port_id_on_b().clone(),
        chan_id_on_b: send_packet_event.chan_id_on_b().clone(),
    };

    // continue packet relay starting from recv_packet at B
    relayer.send_packet_on_a(packet, signer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosts::{MockHost, TendermintHost};

    #[test]
    fn ibc_integration_test_for_all_pairs() {
        ibc_integration_test::<MockHost, MockHost>();
        ibc_integration_test::<MockHost, TendermintHost>();
        ibc_integration_test::<TendermintHost, MockHost>();
        ibc_integration_test::<TendermintHost, TendermintHost>();
    }
}
