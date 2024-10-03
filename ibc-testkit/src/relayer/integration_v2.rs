use ibc::core::channel::handler::chan_register_execute;
use ibc::core::channel::types::channel::Order;
use ibc::core::channel::types::msgs::MsgChannelRegister;
use ibc::core::channel::types::Version;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::host::types::identifiers::{ChannelId, PortId};

use crate::context::TestContext;
use crate::fixtures::core::signer::dummy_account_id;
use crate::hosts::{HostClientState, TestHost};
use crate::relayer::context::RelayerContext;
use crate::testapp::ibc::applications::transfer::types::DummyTransferModule;
use crate::testapp::ibc::core::types::DefaultIbcStore;

/// Integration test for IBC implementation. This test creates clients,
/// connections, channels between two [`TestHost`]s.
///
/// If `serde` feature is enabled, this also exercises packet relay between [`TestHost`]s. This uses
/// [`DummyTransferModule`](crate::testapp::ibc::applications::transfer::types::DummyTransferModule)
/// to simulate the transfer of tokens between two contexts.
pub fn ibc_integration_test<A, B>()
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    let ctx_a = TestContext::<A>::default();
    let ctx_b = TestContext::<B>::default();

    let signer = dummy_account_id();

    let mut relayer = RelayerContext::new(ctx_a, ctx_b);

    // client creation
    let client_id_on_a = relayer.create_client_on_a(signer.clone());
    let client_id_on_b = relayer.create_client_on_b(signer.clone());

    // channel register msg on a
    let msg_on_a = MsgChannelRegister {
        port_id_on_a: PortId::transfer(),
        port_id_on_b: PortId::transfer(),
        client_id_on_a: client_id_on_a.clone(),
        client_id_on_b: client_id_on_b.clone(),
        signer: signer.clone(),
        commitment_prefix_on_b: b"ibc".to_vec().into(),
        ordering: Order::Unordered,
        version_proposal: Version::empty(),
    };

    chan_register_execute(
        relayer.get_ctx_a_mut().ibc_store_mut(),
        &mut DummyTransferModule,
        msg_on_a,
    )
    .expect("channel register should succeed");

    // channel register msg on b
    let msg_on_b = MsgChannelRegister {
        port_id_on_a: PortId::transfer(),
        port_id_on_b: PortId::transfer(),
        client_id_on_a: client_id_on_b.clone(),
        client_id_on_b: client_id_on_a.clone(),
        signer: signer.clone(),
        commitment_prefix_on_b: b"ibc".to_vec().into(),
        ordering: Order::Unordered,
        version_proposal: Version::empty(),
    };

    chan_register_execute(
        relayer.get_ctx_b_mut().ibc_store_mut(),
        &mut DummyTransferModule,
        msg_on_b,
    )
    .expect("channel register should succeed");

    // v2 channels
    let chan_id_on_a = ChannelId::V2(client_id_on_a.clone());
    let _chan_id_on_b = ChannelId::V2(client_id_on_b.clone());

    #[cfg(feature = "serde")]
    {
        use ibc::core::handler::types::events::IbcEvent;

        {
            // ------------------------
            // send packet from A to B
            // ------------------------

            let packet =
                relayer.send_dummy_transfer_packet_on_a(chan_id_on_a.clone(), signer.clone());

            // continue packet relay; submitting recv_packet at B
            relayer.submit_packet_on_b(packet, signer.clone());

            // retrieve the ack_packet event
            let Some(IbcEvent::AcknowledgePacket(_)) = relayer
                .get_ctx_a()
                .ibc_store()
                .events
                .lock()
                .last()
                .cloned()
            else {
                panic!("unexpected event")
            };
        }

        {
            // --------------------------
            // timeout packet from A to B
            // --------------------------

            let packet =
                relayer.send_dummy_transfer_packet_on_a(chan_id_on_a.clone(), signer.clone());

            // timeout the packet on B; by never submitting the packet to B
            relayer.timeout_packet_from_a(packet.clone(), signer.clone());

            // retrieve the timeout_packet event
            let Some(IbcEvent::TimeoutPacket(_)) = relayer
                .get_ctx_a()
                .ibc_store()
                .events
                .lock()
                .last()
                .cloned()
            else {
                panic!("unexpected event")
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosts::{MockHost, TendermintHost};

    // tests among all the `TestHost` implementations
    #[test]
    fn ibc_integration_test_for_all_pairs() {
        ibc_integration_test::<MockHost, MockHost>();
        ibc_integration_test::<MockHost, TendermintHost>();
        ibc_integration_test::<TendermintHost, MockHost>();
        ibc_integration_test::<TendermintHost, TendermintHost>();
    }
}
