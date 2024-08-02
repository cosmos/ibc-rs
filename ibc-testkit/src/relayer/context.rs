use core::fmt::Debug;

use basecoin_store::context::ProvableStore;
use ibc::core::channel::types::packet::Packet;
use ibc::core::client::context::client_state::ClientStateExecution;
use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::host::types::path::ChannelEndPath;
use ibc::core::host::ValidationContext;
use ibc::primitives::proto::Any;
use ibc::primitives::Signer;

use crate::context::StoreGenericTestContext;
use crate::hosts::{HostClientState, HostConsensusState, TestHost};
use crate::relayer::utils::TypedRelayerOps;
use crate::testapp::ibc::core::types::MockIbcStore;

/// A relayer context that allows interaction between two [`StoreGenericTestContext`] instances.
pub struct RelayerContext<A, B, S, ACL, ACS>
where
    A: TestHost,
    B: TestHost,
    S: ProvableStore + Debug,
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
    ctx_a: StoreGenericTestContext<S, A, ACL, ACS>,
    ctx_b: StoreGenericTestContext<S, B, ACL, ACS>,
}

impl<A, B, S, ACL, ACS> RelayerContext<A, B, S, ACL, ACS>
where
    A: TestHost,
    B: TestHost,
    S: ProvableStore + Debug,
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
    /// Creates a new relayer context with the given [`StoreGenericTestContext`] instances.
    pub fn new(
        ctx_a: StoreGenericTestContext<S, A, ACL, ACS>,
        ctx_b: StoreGenericTestContext<S, B, ACL, ACS>,
    ) -> Self {
        Self { ctx_a, ctx_b }
    }

    /// Returns an immutable reference to the first context.
    pub fn get_ctx_a(&self) -> &StoreGenericTestContext<S, A, ACL, ACS> {
        &self.ctx_a
    }

    /// Returns an immutable reference to the second context.
    pub fn get_ctx_b(&self) -> &StoreGenericTestContext<S, B, ACL, ACS> {
        &self.ctx_b
    }

    /// Returns a mutable reference to the first context.
    pub fn get_ctx_a_mut(&mut self) -> &mut StoreGenericTestContext<S, A, ACL, ACS> {
        &mut self.ctx_a
    }

    /// Returns a mutable reference to the second context.
    pub fn get_ctx_b_mut(&mut self) -> &mut StoreGenericTestContext<S, B, ACL, ACS> {
        &mut self.ctx_b
    }

    /// Creates a light client of second context on the first context.
    /// Returns the client identifier of the created client.
    pub fn create_client_on_a(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<A, B, S, ACL, ACS>::create_client_on_a(
            &mut self.ctx_a,
            &self.ctx_b,
            signer,
        )
    }

    /// Creates a light client of first context on the second context.
    /// Returns the client identifier of the created client.
    pub fn create_client_on_b(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<B, A, S, ACL, ACS>::create_client_on_a(
            &mut self.ctx_b,
            &self.ctx_a,
            signer,
        )
    }

    /// Updates the client on the first context with the latest header of the second context.
    pub fn update_client_on_a_with_sync(&mut self, client_id_on_a: ClientId, signer: Signer) {
        TypedRelayerOps::<A, B, S, ACL, ACS>::update_client_on_a_with_sync(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            signer,
        )
    }

    /// Updates the client on the second context with the latest header of the first context.
    pub fn update_client_on_b_with_sync(&mut self, client_id_on_b: ClientId, signer: Signer) {
        TypedRelayerOps::<B, A, S, ACL, ACS>::update_client_on_a_with_sync(
            &mut self.ctx_b,
            &mut self.ctx_a,
            client_id_on_b,
            signer,
        )
    }

    /// Creates a connection between the two contexts starting from the first context.
    /// Returns the connection identifiers of the created connection ends.
    pub fn create_connection_on_a(
        &mut self,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) -> (ConnectionId, ConnectionId) {
        TypedRelayerOps::<A, B, S, ACL, ACS>::create_connection_on_a(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            client_id_on_b,
            signer,
        )
    }

    /// Creates a connection between the two contexts starting from the second context.
    /// Returns the connection identifiers of the created connection ends.
    pub fn create_connection_on_b(
        &mut self,
        client_id_on_b: ClientId,
        client_id_on_a: ClientId,
        signer: Signer,
    ) -> (ConnectionId, ConnectionId) {
        TypedRelayerOps::<B, A, S, ACL, ACS>::create_connection_on_a(
            &mut self.ctx_b,
            &mut self.ctx_a,
            client_id_on_b,
            client_id_on_a,
            signer,
        )
    }

    /// Creates a channel between the two contexts starting from the first context.
    /// Returns the channel identifiers of the created channel ends.
    pub fn create_channel_on_a(
        &mut self,
        conn_id_on_a: ConnectionId,
        port_id_on_a: PortId,
        conn_id_on_b: ConnectionId,
        port_id_on_b: PortId,
        signer: Signer,
    ) -> (ChannelId, ChannelId) {
        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<A, B, S, ACL, ACS>::create_channel_on_a(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            conn_id_on_a,
            port_id_on_a,
            client_id_on_b,
            conn_id_on_b,
            port_id_on_b,
            signer,
        )
    }

    /// Creates a channel between the two contexts starting from the second context.
    /// Returns the channel identifiers of the created channel ends.
    pub fn create_channel_on_b(
        &mut self,
        conn_id_on_b: ConnectionId,
        port_id_on_b: PortId,
        conn_id_on_a: ConnectionId,
        port_id_on_a: PortId,
        signer: Signer,
    ) -> (ChannelId, ChannelId) {
        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<B, A, S, ACL, ACS>::create_channel_on_a(
            &mut self.ctx_b,
            &mut self.ctx_a,
            client_id_on_b,
            conn_id_on_b,
            port_id_on_b,
            client_id_on_a,
            conn_id_on_a,
            port_id_on_a,
            signer,
        )
    }

    /// Closes a channel between the two contexts starting from the first context.
    pub fn close_channel_on_a(
        &mut self,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        let conn_id_on_a = self
            .ctx_a
            .ibc_store()
            .channel_end(&ChannelEndPath::new(&port_id_on_a, &chan_id_on_a))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let conn_id_on_b = self
            .ctx_b
            .ibc_store()
            .channel_end(&ChannelEndPath::new(&port_id_on_b, &chan_id_on_b))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<A, B, S, ACL, ACS>::close_channel_on_a(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            chan_id_on_a,
            port_id_on_a,
            client_id_on_b,
            chan_id_on_b,
            port_id_on_b,
            signer,
        )
    }

    /// Closes a channel between the two contexts starting from the second context.
    pub fn close_channel_on_b(
        &mut self,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        signer: Signer,
    ) {
        let conn_id_on_b = self
            .ctx_b
            .ibc_store()
            .channel_end(&ChannelEndPath::new(&port_id_on_b, &chan_id_on_b))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let conn_id_on_a = self
            .ctx_a
            .ibc_store()
            .channel_end(&ChannelEndPath::new(&port_id_on_a, &chan_id_on_a))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<B, A, S, ACL, ACS>::close_channel_on_a(
            &mut self.ctx_b,
            &mut self.ctx_a,
            client_id_on_b,
            chan_id_on_b,
            port_id_on_b,
            client_id_on_a,
            chan_id_on_a,
            port_id_on_a,
            signer,
        )
    }

    /// Sends a packet from the first context to the second context by
    /// submitting on receive packet on the second context.
    ///
    /// The IBC packet is created by an IBC application on the first context.
    pub fn submit_packet_on_b(&mut self, packet: Packet, signer: Signer) {
        let conn_id_on_a = self
            .ctx_a
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_a,
                &packet.chan_id_on_a,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let conn_id_on_b = self
            .ctx_b
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_b,
                &packet.chan_id_on_b,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<A, B, S, ACL, ACS>::submit_packet_on_b(
            &mut self.ctx_a,
            &mut self.ctx_b,
            packet,
            client_id_on_a,
            client_id_on_b,
            signer,
        )
    }

    /// Times out a packet from the first context to the second context by waiting
    /// for the timeout period and then sending a timeout packet to the first context.
    ///
    /// The IBC packet is created by an IBC application on the first context.
    pub fn timeout_packet_from_a(&mut self, packet: Packet, signer: Signer) {
        let conn_id_on_a = self
            .ctx_a
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_a,
                &packet.chan_id_on_a,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let conn_id_on_b = self
            .ctx_b
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_b,
                &packet.chan_id_on_b,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<A, B, S, ACL, ACS>::timeout_packet_from_a(
            &mut self.ctx_a,
            &mut self.ctx_b,
            packet,
            client_id_on_a,
            client_id_on_b,
            signer,
        )
    }

    /// Timeouts a packet from the first context to the second context by closing the
    /// corresponding channel is closed and then sending a timeout packet to the first context.
    ///
    /// The IBC packet is created by an IBC application on the first context.
    pub fn timeout_packet_from_a_on_channel_close(&mut self, packet: Packet, signer: Signer) {
        let conn_id_on_a = self
            .ctx_a
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_a,
                &packet.chan_id_on_a,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let conn_id_on_b = self
            .ctx_b
            .ibc_store()
            .channel_end(&ChannelEndPath::new(
                &packet.port_id_on_b,
                &packet.chan_id_on_b,
            ))
            .expect("connection exists")
            .connection_hops()[0]
            .clone();

        let client_id_on_a = self
            .ctx_a
            .ibc_store()
            .connection_end(&conn_id_on_a)
            .expect("connection exists")
            .client_id()
            .clone();

        let client_id_on_b = self
            .ctx_b
            .ibc_store()
            .connection_end(&conn_id_on_b)
            .expect("connection exists")
            .client_id()
            .clone();

        TypedRelayerOps::<A, B, S, ACL, ACS>::timeout_packet_from_a_on_channel_close(
            &mut self.ctx_a,
            &mut self.ctx_b,
            packet,
            client_id_on_a,
            client_id_on_b,
            signer,
        )
    }

    /// Submit a
    /// [`DummyTransferModule`](crate::testapp::ibc::applications::transfer::types::DummyTransferModule)
    /// packet on the first context.
    ///
    /// Requires `serde` feature because of [`ibc::apps::transfer::handler::send_transfer`].
    #[cfg(feature = "serde")]
    pub fn send_dummy_transfer_packet_on_a(
        &mut self,
        chan_id_on_a: ChannelId,
        signer: Signer,
    ) -> Packet {
        use ibc::apps::transfer::handler::send_transfer;
        use ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
        use ibc::apps::transfer::types::packet::PacketData;
        use ibc::core::handler::types::events::IbcEvent;
        use ibc::primitives::Timestamp;

        use crate::testapp::ibc::applications::transfer::types::DummyTransferModule;

        // generate packet for DummyTransferModule
        let packet_data = PacketData {
            token: "1000uibc".parse().expect("valid prefixed coin"),
            sender: signer.clone(),
            receiver: signer.clone(),
            memo: "sample memo".into(),
        };

        // packet with ibc metadata
        // either height timeout or timestamp timeout must be set
        let msg = MsgTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: chan_id_on_a.clone(),
            packet_data,
            // setting timeout height to 10 blocks from B's current height.
            timeout_height_on_b: self.get_ctx_b().latest_height().add(10).into(),
            // not setting timeout timestamp.
            timeout_timestamp_on_b: Timestamp::none(),
        };

        // module creates the send_packet
        send_transfer(
            self.get_ctx_a_mut().ibc_store_mut(),
            &mut DummyTransferModule,
            msg,
        )
        .expect("successfully created send_packet");

        // send_packet wasn't committed, hence produce a block
        self.get_ctx_a_mut().advance_block_height();

        // retrieve the send_packet event
        let Some(IbcEvent::SendPacket(send_packet_event)) = self
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
        Packet {
            port_id_on_a: send_packet_event.port_id_on_a().clone(),
            chan_id_on_a: send_packet_event.chan_id_on_a().clone(),
            seq_on_a: *send_packet_event.seq_on_a(),
            data: send_packet_event.packet_data().to_vec(),
            timeout_height_on_b: *send_packet_event.timeout_height_on_b(),
            timeout_timestamp_on_b: *send_packet_event.timeout_timestamp_on_b(),
            port_id_on_b: send_packet_event.port_id_on_b().clone(),
            chan_id_on_b: send_packet_event.chan_id_on_b().clone(),
        }
    }
}
