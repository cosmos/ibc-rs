use alloc::string::String;
use core::marker::PhantomData;
use core::time::Duration;

use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgUpdateClient};
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

use crate::context::MockContext;
use crate::hosts::{HostClientState, TestBlock, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::{DefaultIbcStore, LightClientBuilder, LightClientState};

#[derive(Debug, Default)]
pub struct TypedRelayer<A, B>(PhantomData<A>, PhantomData<B>)
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>;

impl<A, B> TypedRelayer<A, B>
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>,
{
    pub fn create_client_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        signer: Signer,
    ) -> ClientId {
        let light_client_of_b = LightClientBuilder::init()
            .context(ctx_b)
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

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::CreateClient(create_client_b_event)) =
            ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        let client_id_on_a = create_client_b_event.client_id().clone();

        assert_eq!(
            ValidationContext::get_client_validation_context(ctx_a.ibc_store())
                .client_state(&client_id_on_a)
                .expect("client state exists")
                .latest_height(),
            ctx_b.latest_height()
        );

        client_id_on_a
    }

    pub fn sync_latest_timestamp(ctx_a: &mut MockContext<A>, ctx_b: &mut MockContext<B>) {
        if ctx_a.latest_timestamp() > ctx_b.latest_timestamp() {
            while ctx_a.latest_timestamp() > ctx_b.latest_timestamp() {
                ctx_b.advance_block();
            }
        } else {
            while ctx_b.latest_timestamp() > ctx_a.latest_timestamp() {
                ctx_a.advance_block();
            }
        }
    }

    pub fn update_client_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        client_id_on_a: ClientId,
        signer: Signer,
    ) {
        let latest_client_height_on_a = ctx_a
            .ibc_store()
            .get_client_validation_context()
            .client_state(&client_id_on_a)
            .expect("client state exists")
            .latest_height();

        let latest_height_of_b = ctx_b.latest_height();

        let msg_for_a = MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
            client_id: client_id_on_a.clone(),
            client_message: ctx_b
                .host_block(&latest_height_of_b)
                .expect("block exists")
                .into_header_with_previous_block(
                    &ctx_b
                        .host_block(&latest_client_height_on_a)
                        .expect("block exists"),
                )
                .into(),
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::UpdateClient(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn update_client_on_a_with_sync(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &mut MockContext<B>,
        client_id_on_a: ClientId,
        signer: Signer,
    ) {
        TypedRelayer::<A, B>::sync_latest_timestamp(ctx_a, ctx_b);
        TypedRelayer::<A, B>::update_client_on_a(ctx_a, router_a, ctx_b, client_id_on_a, signer);
    }

    pub fn connection_open_init_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) -> ConnectionId {
        let counterparty_b = ConnectionCounterParty::new(
            client_id_on_b.clone(),
            None,
            ctx_b.ibc_store().commitment_prefix(),
        );

        let msg_for_a = MsgEnvelope::Connection(ConnectionMsg::OpenInit(MsgConnectionOpenInit {
            client_id_on_a: client_id_on_a.clone(),
            counterparty: counterparty_b,
            version: None,
            delay_period: Duration::from_secs(0),
            signer: signer.clone(),
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::OpenInitConnection(open_init_connection_event)) =
            ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        open_init_connection_event.conn_id_on_a().clone()
    }

    pub fn connection_open_try_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        conn_id_on_a: ConnectionId,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) -> ConnectionId {
        let proofs_height_on_a = ctx_a.latest_height();

        let client_state_of_b_on_a = ctx_a
            .ibc_store()
            .client_state(&client_id_on_a)
            .expect("client state exists");

        let consensus_height_of_b_on_a = client_state_of_b_on_a.latest_height();

        let counterparty_a = ConnectionCounterParty::new(
            client_id_on_a.clone(),
            Some(conn_id_on_a.clone()),
            ctx_a.ibc_store().commitment_prefix(),
        );

        let proof_conn_end_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proofs_height_on_a,
                &ConnectionPath::new(&conn_id_on_a).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let proof_client_state_of_b_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proofs_height_on_a,
                &ClientStatePath::new(client_id_on_a.clone()).into(),
            )
            .expect("client state exists")
            .try_into()
            .expect("value merkle proof");

        let proof_consensus_state_of_b_on_a = ctx_a
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

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::OpenTryConnection(open_try_connection_event)) =
            ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        open_try_connection_event.conn_id_on_b().clone()
    }

    pub fn connection_open_ack_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        conn_id_on_a: ConnectionId,
        conn_id_on_b: ConnectionId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) {
        let proofs_height_on_b = ctx_b.latest_height();

        let client_state_of_a_on_b = ctx_b
            .ibc_store()
            .client_state(&client_id_on_b)
            .expect("client state exists");

        let consensus_height_of_a_on_b = client_state_of_a_on_b.latest_height();

        let proof_conn_end_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proofs_height_on_b,
                &ConnectionPath::new(&conn_id_on_b).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let proof_client_state_of_a_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proofs_height_on_b,
                &ClientStatePath::new(client_id_on_b.clone()).into(),
            )
            .expect("client state exists")
            .try_into()
            .expect("value merkle proof");

        let proof_consensus_state_of_a_on_b = ctx_b
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

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::OpenAckConnection(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn connection_open_confirm_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        conn_id_on_a: ConnectionId,
        conn_id_on_b: ConnectionId,
        signer: Signer,
    ) {
        let proof_height_on_a = ctx_a.latest_height();

        let proof_conn_end_on_a = ctx_a
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

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::OpenConfirmConnection(_)) = ctx_b.ibc_store().events.lock().last()
        else {
            panic!("unexpected event")
        };
    }

    pub fn create_connection_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) -> (ConnectionId, ConnectionId) {
        let conn_id_on_a = TypedRelayer::<A, B>::connection_open_init_on_a(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedRelayer::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        let conn_id_on_b = TypedRelayer::<A, B>::connection_open_try_on_b(
            ctx_b,
            router_b,
            ctx_a,
            conn_id_on_a.clone(),
            client_id_on_a.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedRelayer::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a.clone(),
            signer.clone(),
        );

        TypedRelayer::<A, B>::connection_open_ack_on_a(
            ctx_a,
            router_a,
            ctx_b,
            conn_id_on_a.clone(),
            conn_id_on_b.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedRelayer::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedRelayer::<A, B>::connection_open_confirm_on_b(
            ctx_b,
            router_b,
            ctx_a,
            conn_id_on_b.clone(),
            conn_id_on_a.clone(),
            signer.clone(),
        );

        TypedRelayer::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer,
        );

        (conn_id_on_a, conn_id_on_b)
    }
}
