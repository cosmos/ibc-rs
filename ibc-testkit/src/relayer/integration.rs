use core::fmt::Debug;

use basecoin_store::context::ProvableStore;
use ibc::core::client::context::client_state::ClientStateExecution;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc::primitives::proto::Any;

use crate::context::StoreGenericTestContext;
use crate::fixtures::core::signer::dummy_account_id;
use crate::hosts::{HostClientState, HostConsensusState, TestHost};
use crate::relayer::context::RelayerContext;
use crate::testapp::ibc::core::types::MockIbcStore;
use ibc::core::client::context::consensus_state::ConsensusState;

/// Integration test for IBC implementation. This test creates clients,
/// connections, channels between two [`TestHost`]s.
///
/// If `serde` feature is enabled, this also exercises packet relay between [`TestHost`]s. This uses
/// [`DummyTransferModule`](crate::testapp::ibc::applications::transfer::types::DummyTransferModule)
/// to simulate the transfer of tokens between two contexts.
pub fn ibc_integration_test<A, B, S, ACL, ACS>()
where
    A: TestHost,
    B: TestHost,
    S: ProvableStore + Debug + Default,
    ACL: From<HostClientState<A>>
        + From<HostClientState<B>>
        + ClientStateExecution<MockIbcStore<S, ACL, ACS>>
        + Clone,
    ACS: From<HostConsensusState<A>> + From<HostConsensusState<B>> + ConsensusState + Clone,
    HostClientState<A>: ClientStateExecution<MockIbcStore<S, ACL, ACS>>,
    HostClientState<B>: ClientStateExecution<MockIbcStore<S, ACL, ACS>>,
    MockIbcStore<S, ACL, ACS>:
        ClientExecutionContext<ClientStateMut = ACL, ConsensusStateRef = ACS>,
    ClientError: From<<ACL as TryFrom<Any>>::Error>,
{
    let ctx_a = StoreGenericTestContext::<S, A, ACL, ACS>::default();
    let ctx_b = StoreGenericTestContext::<S, B, ACL, ACS>::default();

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

        {
            // ------------------------------------------------
            // timeout packet from A to B; using closed channel
            // ------------------------------------------------

            let packet =
                relayer.send_dummy_transfer_packet_on_a(chan_id_on_a.clone(), signer.clone());

            // timeout the packet on B; close the corresponding channel
            relayer.timeout_packet_from_a_on_channel_close(packet.clone(), signer.clone());

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
    use crate::context::MockStore;
    use crate::hosts::{MockHost, TendermintHost};
    use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
    use crate::testapp::ibc::core::types::DefaultIbcStore;

    // tests among all the `TestHost` implementations
    #[test]
    fn ibc_integration_test_for_all_pairs() {
        fn default_ibc_integration_test<A, B>()
        where
            A: TestHost,
            B: TestHost,
            AnyClientState: From<HostClientState<A>> + From<HostClientState<B>>,
            AnyConsensusState: From<HostConsensusState<A>> + From<HostConsensusState<B>>,
            HostClientState<A>: ClientStateExecution<DefaultIbcStore>,
            HostClientState<B>: ClientStateExecution<DefaultIbcStore>,
        {
            ibc_integration_test::<A, B, MockStore, AnyClientState, AnyConsensusState>();
        }

        default_ibc_integration_test::<MockHost, MockHost>();
        default_ibc_integration_test::<MockHost, TendermintHost>();
        default_ibc_integration_test::<TendermintHost, MockHost>();
        default_ibc_integration_test::<TendermintHost, TendermintHost>();
    }
}
