use ibc::core::channel::types::packet::Packet;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::host::types::path::ChannelEndPath;
use ibc::core::host::ValidationContext;
use ibc::primitives::Signer;

use crate::context::TestContext;
use crate::hosts::{HostClientState, TestHost};
use crate::relayer::utils::TypedRelayerOps;
use crate::testapp::ibc::core::types::DefaultIbcStore;

/// A relayer context that allows interaction between two [`TestContext`] instances.
pub struct RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    ctx_a: TestContext<A>,
    ctx_b: TestContext<B>,
}

impl<A, B> RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    /// Creates a new relayer context with the given [`TestContext`] instances.
    pub fn new(ctx_a: TestContext<A>, ctx_b: TestContext<B>) -> Self {
        Self { ctx_a, ctx_b }
    }

    /// Returns immutable reference to the first context.
    pub fn get_ctx_a(&self) -> &TestContext<A> {
        &self.ctx_a
    }

    /// Returns immutable reference to the second context.
    pub fn get_ctx_b(&self) -> &TestContext<B> {
        &self.ctx_b
    }

    /// Returns mutable reference to the first context.
    pub fn get_ctx_a_mut(&mut self) -> &mut TestContext<A> {
        &mut self.ctx_a
    }

    /// Returns mutable reference to the second context.
    pub fn get_ctx_b_mut(&mut self) -> &mut TestContext<B> {
        &mut self.ctx_b
    }

    /// Creates a light client of second context on the first context.
    /// Returns the client identifier of the created client.
    pub fn create_client_on_a(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<A, B>::create_client_on_a(&mut self.ctx_a, &self.ctx_b, signer)
    }

    /// Creates a light client of first context on the second context.
    /// Returns the client identifier of the created client.
    pub fn create_client_on_b(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<B, A>::create_client_on_a(&mut self.ctx_b, &self.ctx_a, signer)
    }

    /// Updates the client on the first context with the latest header of the second context.
    pub fn update_client_on_a_with_sync(&mut self, client_id_on_a: ClientId, signer: Signer) {
        TypedRelayerOps::<A, B>::update_client_on_a_with_sync(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            signer,
        )
    }

    /// Updates the client on the second context with the latest header of the first context.
    pub fn update_client_on_b_with_sync(&mut self, client_id_on_b: ClientId, signer: Signer) {
        TypedRelayerOps::<B, A>::update_client_on_a_with_sync(
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
        TypedRelayerOps::<A, B>::create_connection_on_a(
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
        TypedRelayerOps::<B, A>::create_connection_on_a(
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

        TypedRelayerOps::<A, B>::create_channel_on_a(
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

        TypedRelayerOps::<B, A>::create_channel_on_a(
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

        TypedRelayerOps::<A, B>::close_channel_on_a(
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

        TypedRelayerOps::<B, A>::close_channel_on_a(
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

    /// Sends a packet from the first context to the second context.
    /// The IBC packet is created by an IBC application on the first context.
    pub fn send_packet_on_a(&mut self, packet: Packet, signer: Signer) {
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

        TypedRelayerOps::<A, B>::send_packet_on_a(
            &mut self.ctx_a,
            &mut self.ctx_b,
            packet,
            client_id_on_a,
            client_id_on_b,
            signer,
        )
    }
}
