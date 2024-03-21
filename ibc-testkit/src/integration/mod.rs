use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::primitives::Signer;

use self::utils::TypedRelayer;
use crate::context::MockContext;
use crate::fixtures::core::signer::dummy_account_id;
use crate::hosts::{HostClientState, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::DefaultIbcStore;

pub mod utils;

pub struct RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    ctx_a: MockContext<A>,
    router_a: MockRouter,
    ctx_b: MockContext<B>,
    router_b: MockRouter,
}

impl<A, B> RelayerContext<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    pub fn new(
        ctx_a: MockContext<A>,
        router_a: MockRouter,
        ctx_b: MockContext<B>,
        router_b: MockRouter,
    ) -> Self {
        Self {
            ctx_a,
            router_a,
            ctx_b,
            router_b,
        }
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
        TypedRelayer::<A, B>::create_client_on_a(
            &mut self.ctx_a,
            &mut self.router_a,
            &self.ctx_b,
            signer,
        )
    }

    pub fn create_client_on_b(&mut self, signer: Signer) -> ClientId {
        TypedRelayer::<B, A>::create_client_on_a(
            &mut self.ctx_b,
            &mut self.router_b,
            &self.ctx_a,
            signer,
        )
    }

    pub fn update_client_on_a_with_sync(&mut self, client_id_on_a: ClientId, signer: Signer) {
        TypedRelayer::<A, B>::update_client_on_a_with_sync(
            &mut self.ctx_a,
            &mut self.router_a,
            &mut self.ctx_b,
            client_id_on_a,
            signer,
        )
    }

    pub fn update_client_on_b_with_sync(&mut self, client_id_on_b: ClientId, signer: Signer) {
        TypedRelayer::<B, A>::update_client_on_a_with_sync(
            &mut self.ctx_b,
            &mut self.router_b,
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
        TypedRelayer::<A, B>::create_connection_on_a(
            &mut self.ctx_a,
            &mut self.router_a,
            &mut self.ctx_b,
            &mut self.router_b,
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
        TypedRelayer::<B, A>::create_connection_on_a(
            &mut self.ctx_b,
            &mut self.router_b,
            &mut self.ctx_a,
            &mut self.router_a,
            client_id_on_b,
            client_id_on_a,
            signer,
        )
    }
}

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

    let mut relayer =
        RelayerContext::new(ctx_a, MockRouter::default(), ctx_b, MockRouter::default());

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
    let (conn_id_on_b, conn_id_on_a) =
        relayer.create_connection_on_b(client_id_on_b, client_id_on_a, signer);

    assert_eq!(conn_id_on_a, ConnectionId::new(1));
    assert_eq!(conn_id_on_b, ConnectionId::new(1));

    // channel/packet integration; test timeouts
    // TODO(rano): test channel and packet messages require a module
    // create integration with ics-20 module
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
