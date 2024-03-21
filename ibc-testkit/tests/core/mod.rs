use std::time::Duration;

use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgUpdateClient};
use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::{
    ConnectionMsg, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry,
};
use ibc::core::connection::types::version::Version;
use ibc::core::connection::types::Counterparty as ConnectionCounterParty;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::core::host::types::path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath};
use ibc::core::host::ValidationContext;
use ibc::primitives::Signer;
use ibc_query::core::context::ProvableContext;
use ibc_testkit::context::MockContext;
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::hosts::{HostClientState, TendermintHost, TestBlock, TestHost};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{
    DefaultIbcStore, LightClientBuilder, LightClientState,
};

pub mod ics02_client;
pub mod ics03_connection;
pub mod ics04_channel;
#[cfg(feature = "serde")]
pub mod router;

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

    pub fn reverse(self) -> RelayerContext<B, A> {
        RelayerContext::new(self.ctx_b, self.router_b, self.ctx_a, self.router_a)
    }

    pub fn create_client_on_a(&mut self, signer: Signer) -> ClientId {
        let light_client_of_b = LightClientBuilder::init()
            .context(&self.ctx_b)
            .build::<LightClientState<B>>();

        let msg_for_a = MsgEnvelope::Client(ClientMsg::CreateClient(MsgCreateClient {
            client_state: light_client_of_b.client_state.into(),
            consensus_state: light_client_of_b
                .consensus_states
                .values()
                .next()
                .expect("at least one")
                .clone()
                .into()
                .into(),
            signer,
        }));

        self.ctx_a
            .deliver(&mut self.router_a, msg_for_a)
            .expect("success");

        let Some(IbcEvent::CreateClient(create_client_b_event)) =
            self.ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        let client_id_on_a = create_client_b_event.client_id().clone();

        assert_eq!(
            ValidationContext::get_client_validation_context(self.ctx_a.ibc_store())
                .client_state(&client_id_on_a)
                .expect("client state exists")
                .latest_height(),
            self.ctx_b.latest_height()
        );

        client_id_on_a
    }

    pub fn sync_latest_timestamp(&mut self) {
        if self.ctx_a.latest_timestamp() > self.ctx_b.latest_timestamp() {
            while self.ctx_a.latest_timestamp() > self.ctx_b.latest_timestamp() {
                self.ctx_b.advance_block();
            }
        } else {
            while self.ctx_b.latest_timestamp() > self.ctx_a.latest_timestamp() {
                self.ctx_a.advance_block();
            }
        }
    }

    pub fn update_client_on_a(&mut self, client_id_on_a: ClientId, signer: Signer) {
        let latest_client_height_on_a = self
            .ctx_a
            .ibc_store()
            .get_client_validation_context()
            .client_state(&client_id_on_a)
            .unwrap()
            .latest_height();

        let latest_height_of_b = self.ctx_b.latest_height();

        let msg_for_a = MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
            client_id: client_id_on_a.clone(),
            client_message: self
                .ctx_b
                .host_block(&latest_height_of_b)
                .expect("block exists")
                .into_header_with_previous_block(
                    &self
                        .ctx_b
                        .host_block(&latest_client_height_on_a)
                        .expect("block exists"),
                )
                .into(),
            signer,
        }));

        self.sync_latest_timestamp();

        self.ctx_a
            .deliver(&mut self.router_a, msg_for_a)
            .expect("success");

        let Some(IbcEvent::UpdateClient(_)) = self.ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn update_client_on_b(&mut self, client_id_on_b: ClientId, signer: Signer) {
        let latest_client_height_on_b = self
            .ctx_b
            .ibc_store()
            .get_client_validation_context()
            .client_state(&client_id_on_b)
            .unwrap()
            .latest_height();

        let latest_height_of_a = self.ctx_a.latest_height();

        let msg_for_b = MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
            client_id: client_id_on_b.clone(),
            client_message: self
                .ctx_a
                .host_block(&latest_height_of_a)
                .expect("block exists")
                .into_header_with_previous_block(
                    &self
                        .ctx_a
                        .host_block(&latest_client_height_on_b)
                        .expect("block exists"),
                )
                .into(),
            signer,
        }));

        self.sync_latest_timestamp();

        self.ctx_b
            .deliver(&mut self.router_b, msg_for_b)
            .expect("success");

        let Some(IbcEvent::UpdateClient(_)) = self.ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn create_connection_on_b(
        &mut self,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) -> (ConnectionId, ConnectionId) {
        let counterparty_b = ConnectionCounterParty::new(
            client_id_on_b.clone(),
            None,
            self.ctx_b.ibc_store().commitment_prefix(),
        );

        let msg_for_a = MsgEnvelope::Connection(ConnectionMsg::OpenInit(MsgConnectionOpenInit {
            client_id_on_a: client_id_on_a.clone(),
            counterparty: counterparty_b,
            version: None,
            delay_period: Duration::from_secs(0),
            signer: signer.clone(),
        }));

        self.ctx_a
            .deliver(&mut self.router_a, msg_for_a)
            .expect("success");

        let Some(IbcEvent::OpenInitConnection(open_init_connection_event)) =
            self.ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
        self.update_client_on_b(client_id_on_b.clone(), signer.clone());

        let conn_id_on_a = open_init_connection_event.conn_id_on_a().clone();

        let proofs_height_on_a = self.ctx_a.latest_height();

        let client_state_of_b_on_a = self
            .ctx_a
            .ibc_store()
            .client_state(&client_id_on_a)
            .unwrap();

        let consensus_height_of_b_on_a = client_state_of_b_on_a.latest_height();

        let counterparty_a = ConnectionCounterParty::new(
            client_id_on_a.clone(),
            Some(conn_id_on_a.clone()),
            self.ctx_a.ibc_store().commitment_prefix(),
        );

        let proof_conn_end_on_a = self
            .ctx_a
            .ibc_store()
            .get_proof(
                proofs_height_on_a,
                &ConnectionPath::new(&conn_id_on_a).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let proof_client_state_of_b_on_a = self
            .ctx_a
            .ibc_store()
            .get_proof(
                proofs_height_on_a,
                &ClientStatePath::new(client_id_on_a.clone()).into(),
            )
            .expect("client state exists")
            .try_into()
            .expect("value merkle proof");

        let proof_consensus_state_of_b_on_a = self
            .ctx_a
            .ibc_store()
            .get_proof(
                proofs_height_on_a,
                &ClientConsensusStatePath::new(
                    client_id_on_a.clone(),
                    consensus_height_of_b_on_a.revision_number(),
                    consensus_height_of_b_on_a.revision_height(),
                )
                .into(),
            )
            .expect("consensus state exists")
            .try_into()
            .expect("value merkle proof");

        #[allow(deprecated)]
        let msg_for_b = MsgEnvelope::Connection(ConnectionMsg::OpenTry(MsgConnectionOpenTry {
            client_id_on_b: client_id_on_b.clone(),
            client_state_of_b_on_a: client_state_of_b_on_a.into(),
            counterparty: counterparty_a,
            versions_on_a: Version::compatibles(),
            proof_conn_end_on_a,
            proof_client_state_of_b_on_a,
            proof_consensus_state_of_b_on_a,
            proofs_height_on_a,
            consensus_height_of_b_on_a,
            delay_period: Duration::from_secs(0),
            signer: signer.clone(),
            proof_consensus_state_of_b: None,
            // deprecated
            previous_connection_id: String::new(),
        }));

        self.ctx_b
            .deliver(&mut self.router_b, msg_for_b)
            .expect("success");

        let Some(IbcEvent::OpenTryConnection(open_try_connection_event)) =
            self.ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
        self.update_client_on_a(client_id_on_a.clone(), signer.clone());

        let conn_id_on_b = open_try_connection_event.conn_id_on_b().clone();
        let proofs_height_on_b = self.ctx_b.latest_height();

        let client_state_of_a_on_b = self
            .ctx_b
            .ibc_store()
            .client_state(&client_id_on_b)
            .unwrap();

        let consensus_height_of_a_on_b = client_state_of_a_on_b.latest_height();

        let proof_conn_end_on_b = self
            .ctx_b
            .ibc_store()
            .get_proof(
                proofs_height_on_b,
                &ConnectionPath::new(&conn_id_on_b).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let proof_client_state_of_a_on_b = self
            .ctx_b
            .ibc_store()
            .get_proof(
                proofs_height_on_b,
                &ClientStatePath::new(client_id_on_b.clone()).into(),
            )
            .expect("client state exists")
            .try_into()
            .expect("value merkle proof");

        let proof_consensus_state_of_a_on_b = self
            .ctx_b
            .ibc_store()
            .get_proof(
                proofs_height_on_b,
                &ClientConsensusStatePath::new(
                    client_id_on_b.clone(),
                    consensus_height_of_a_on_b.revision_number(),
                    consensus_height_of_a_on_b.revision_height(),
                )
                .into(),
            )
            .expect("consensus state exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_a = MsgEnvelope::Connection(ConnectionMsg::OpenAck(MsgConnectionOpenAck {
            conn_id_on_a: conn_id_on_a.clone(),
            conn_id_on_b: conn_id_on_b.clone(),
            client_state_of_a_on_b: client_state_of_a_on_b.into(),
            proof_conn_end_on_b,
            proof_client_state_of_a_on_b,
            proof_consensus_state_of_a_on_b,
            proofs_height_on_b,
            consensus_height_of_a_on_b,
            version: Version::compatibles()[0].clone(),
            signer: signer.clone(),
            proof_consensus_state_of_a: None,
        }));

        self.ctx_a
            .deliver(&mut self.router_a, msg_for_a)
            .expect("success");

        let Some(IbcEvent::OpenAckConnection(_)) =
            self.ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
        self.update_client_on_b(client_id_on_b.clone(), signer.clone());

        let proof_height_on_a = self.ctx_a.latest_height();

        let proof_conn_end_on_a = self
            .ctx_a
            .ibc_store()
            .get_proof(
                proof_height_on_a,
                &ConnectionPath::new(&conn_id_on_a).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_b =
            MsgEnvelope::Connection(ConnectionMsg::OpenConfirm(MsgConnectionOpenConfirm {
                conn_id_on_b: conn_id_on_b.clone(),
                proof_conn_end_on_a,
                proof_height_on_a,
                signer: signer.clone(),
            }));

        self.ctx_b
            .deliver(&mut self.router_b, msg_for_b)
            .expect("success");

        let Some(IbcEvent::OpenConfirmConnection(_)) = self.ctx_b.ibc_store().events.lock().last()
        else {
            panic!("unexpected event")
        };
        self.update_client_on_a(client_id_on_a.clone(), signer.clone());

        (conn_id_on_a, conn_id_on_b)
    }
}

pub fn integration_test_between_host_pair<P, Q>()
where
    P: TestHost,
    Q: TestHost,
    HostClientState<P>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<Q>: ClientStateValidation<DefaultIbcStore>,
{
    let ctx_a = MockContext::<P>::default();
    let ctx_b = MockContext::<Q>::default();

    let signer = dummy_account_id();

    let mut relayer =
        RelayerContext::new(ctx_a, MockRouter::default(), ctx_b, MockRouter::default());

    assert_eq!(
        relayer.get_ctx_a().latest_height(),
        Height::new(0, 5).expect("no error")
    );
    assert_eq!(
        relayer.get_ctx_b().latest_height(),
        Height::new(0, 5).expect("no error")
    );

    // client q on context p
    let client_id_on_a = relayer.create_client_on_a(signer.clone());

    let mut relayer = relayer.reverse();

    // client p on context q
    let client_id_on_b = relayer.create_client_on_a(signer.clone());

    let mut relayer = relayer.reverse();

    // asserts

    assert_eq!(
        relayer.get_ctx_a().latest_height(),
        Height::new(0, 6).expect("no error")
    );
    assert_eq!(
        relayer.get_ctx_b().latest_height(),
        Height::new(0, 6).expect("no error")
    );

    // connection
    let (_conn_id_on_a, _conn_id_on_b) =
        relayer.create_connection_on_b(client_id_on_a, client_id_on_b, signer);

    // channel

    // channel/packet timeout
}

#[test]
fn tendermint_integration_test() {
    integration_test_between_host_pair::<TendermintHost, TendermintHost>();
}
