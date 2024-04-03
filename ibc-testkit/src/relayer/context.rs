use ibc::core::channel::types::packet::Packet;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::host::types::path::ChannelEndPath;
use ibc::core::host::ValidationContext;
use ibc::primitives::Signer;

use crate::context::MockContext;
use crate::hosts::{HostClientState, TestHost};
use crate::relayer::utils::TypedRelayerOps;
use crate::testapp::ibc::core::types::DefaultIbcStore;

pub struct RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    ctx_a: MockContext<A>,
    ctx_b: MockContext<B>,
}

impl<A, B> RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    pub fn new(ctx_a: MockContext<A>, ctx_b: MockContext<B>) -> Self {
        Self { ctx_a, ctx_b }
    }

    pub fn get_ctx_a(&self) -> &MockContext<A> {
        &self.ctx_a
    }

    pub fn get_ctx_b(&self) -> &MockContext<B> {
        &self.ctx_b
    }

    pub fn get_ctx_a_mut(&mut self) -> &mut MockContext<A> {
        &mut self.ctx_a
    }

    pub fn get_ctx_b_mut(&mut self) -> &mut MockContext<B> {
        &mut self.ctx_b
    }

    pub fn create_client_on_a(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<A, B>::create_client_on_a(&mut self.ctx_a, &self.ctx_b, signer)
    }

    pub fn create_client_on_b(&mut self, signer: Signer) -> ClientId {
        TypedRelayerOps::<B, A>::create_client_on_a(&mut self.ctx_b, &self.ctx_a, signer)
    }

    pub fn update_client_on_a_with_sync(&mut self, client_id_on_a: ClientId, signer: Signer) {
        TypedRelayerOps::<A, B>::update_client_on_a_with_sync(
            &mut self.ctx_a,
            &mut self.ctx_b,
            client_id_on_a,
            signer,
        )
    }

    pub fn update_client_on_b_with_sync(&mut self, client_id_on_b: ClientId, signer: Signer) {
        TypedRelayerOps::<B, A>::update_client_on_a_with_sync(
            &mut self.ctx_b,
            &mut self.ctx_a,
            client_id_on_b,
            signer,
        )
    }

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

#[cfg(test)]
mod tests {
    use ibc::clients::tendermint::types::client_type as tm_client_type;
    use ibc::core::client::context::client_state::ClientStateValidation;
    use ibc::core::client::types::msgs::{ClientMsg, MsgUpdateClient};
    use ibc::core::client::types::Height;
    use ibc::core::handler::types::msgs::MsgEnvelope;
    use ibc::core::host::types::identifiers::{ChainId, ClientId};
    use ibc::core::primitives::prelude::*;
    use tracing::debug;

    use crate::context::MockContext;
    use crate::fixtures::core::context::MockContextConfig;
    use crate::fixtures::core::signer::dummy_account_id;
    use crate::hosts::{
        HostClientState, MockHost, TendermintHost, TestBlock, TestHeader, TestHost,
    };
    use crate::relayer::error::RelayerError;
    use crate::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
    use crate::testapp::ibc::core::types::{DefaultIbcStore, LightClientBuilder};

    /// Builds a `ClientMsg::UpdateClient` for a client with id `client_id` running on the `dest`
    /// context, assuming that the latest header on the source context is `src_header`.
    pub(crate) fn build_client_update_datagram<H: TestHeader, Dst: TestHost>(
        dest: &MockContext<Dst>,
        client_id: &ClientId,
        src_header: &H,
    ) -> Result<ClientMsg, RelayerError>
    where
        HostClientState<Dst>: ClientStateValidation<DefaultIbcStore>,
    {
        // Check if client for ibc0 on ibc1 has been updated to latest height:
        // - query client state on destination chain
        let dest_client_latest_height = dest.light_client_latest_height(client_id);

        if src_header.height() == dest_client_latest_height {
            return Err(RelayerError::ClientAlreadyUpToDate {
                client_id: client_id.clone(),
                source_height: src_header.height(),
                destination_height: dest_client_latest_height,
            });
        };

        if dest_client_latest_height > src_header.height() {
            return Err(RelayerError::ClientAtHigherHeight {
                client_id: client_id.clone(),
                source_height: src_header.height(),
                destination_height: dest_client_latest_height,
            });
        };

        // Client on destination chain can be updated.
        Ok(ClientMsg::UpdateClient(MsgUpdateClient {
            client_id: client_id.clone(),
            client_message: src_header.clone().into(),
            signer: dummy_account_id(),
        }))
    }

    #[test]
    /// Serves to test both ICS-26 `dispatch` & `build_client_update_datagram` functions.
    /// Implements a "ping pong" of client update messages, so that two chains repeatedly
    /// process a client update message and update their height in succession.
    fn client_update_ping_pong() {
        let chain_a_start_height = Height::new(1, 11).unwrap();
        let chain_b_start_height = Height::new(1, 20).unwrap();
        let client_on_b_for_a_height = Height::new(1, 10).unwrap(); // Should be smaller than `chain_a_start_height`
        let client_on_a_for_b_height = Height::new(1, 20).unwrap(); // Should be smaller than `chain_b_start_height`
        let num_iterations = 4;

        let client_on_a_for_b = tm_client_type().build_client_id(0);
        let client_on_b_for_a = mock_client_type().build_client_id(0);

        let chain_id_a = ChainId::new("mockgaiaA-1").unwrap();
        let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

        // Create two mock contexts, one for each chain.
        let mut ctx_a = MockContextConfig::builder()
            .host(MockHost::builder().chain_id(chain_id_a).build())
            .latest_height(chain_a_start_height)
            .build::<MockContext<MockHost>>();

        let mut ctx_b = MockContextConfig::builder()
            .host(TendermintHost::builder().chain_id(chain_id_b).build())
            .latest_height(chain_b_start_height)
            .latest_timestamp(ctx_a.timestamp_at(chain_a_start_height.decrement().unwrap())) // chain B is running slower than chain A
            .build::<MockContext<TendermintHost>>();

        ctx_a = ctx_a.with_light_client(
            &client_on_a_for_b,
            LightClientBuilder::init()
                .context(&ctx_b)
                .consensus_heights([client_on_a_for_b_height])
                .build(),
        );

        ctx_b = ctx_b.with_light_client(
            &client_on_b_for_a,
            LightClientBuilder::init()
                .context(&ctx_a)
                .consensus_heights([client_on_b_for_a_height])
                .build(),
        );

        for _i in 0..num_iterations {
            // Update client on chain B to latest height of A.
            // - create the client update message with the latest header from A
            let a_latest_header = ctx_a.query_latest_block().unwrap();
            let client_msg_b_res = build_client_update_datagram(
                &ctx_b,
                &client_on_b_for_a,
                &a_latest_header.into_header(),
            );

            assert!(
                client_msg_b_res.is_ok(),
                "create_client_update failed for context destination {ctx_b:?}, error: {client_msg_b_res:?}",
            );

            let client_msg_b = client_msg_b_res.unwrap();

            // - send the message to B. We bypass ICS18 interface and call directly into
            // MockContext `recv` method (to avoid additional serialization steps).
            let dispatch_res_b = ctx_b.deliver(MsgEnvelope::Client(client_msg_b));
            let validation_res = ctx_b.host.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {validation_res:?} for context {ctx_b:?}",
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_b.is_ok(),
                "Dispatch failed for host chain b with error: {dispatch_res_b:?}"
            );

            assert_eq!(
                ctx_b.light_client_latest_height(&client_on_b_for_a),
                ctx_a.latest_height()
            );

            // Update client on chain A to latest height of B.
            // - create the client update message with the latest header from B
            // The test uses LightClientBlock that does not store the trusted height
            let mut b_latest_header = ctx_b.query_latest_block().unwrap().clone().into_header();

            let th = b_latest_header.height();
            b_latest_header.set_trusted_height(th.decrement().unwrap());

            let client_msg_a_res =
                build_client_update_datagram(&ctx_a, &client_on_a_for_b, &b_latest_header);

            assert!(
                client_msg_a_res.is_ok(),
                "create_client_update failed for context destination {ctx_a:?}, error: {client_msg_a_res:?}",
            );

            let client_msg_a = client_msg_a_res.unwrap();

            debug!("client_msg_a = {:?}", client_msg_a);

            // - send the message to A
            let dispatch_res_a = ctx_a.deliver(MsgEnvelope::Client(client_msg_a));
            let validation_res = ctx_a.host.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {validation_res:?} for context {ctx_a:?}",
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_a.is_ok(),
                "Dispatch failed for host chain a with error: {dispatch_res_a:?}"
            );
            assert_eq!(
                ctx_a.light_client_latest_height(&client_on_a_for_b),
                ctx_b.latest_height()
            );
        }
    }
}
