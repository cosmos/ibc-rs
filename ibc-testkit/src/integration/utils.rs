use alloc::string::String;
use core::marker::PhantomData;
use core::time::Duration;

use ibc::core::channel::types::acknowledgement::Acknowledgement;
use ibc::core::channel::types::channel::Order;
use ibc::core::channel::types::msgs::{
    ChannelMsg, MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
    MsgTimeoutOnClose, PacketMsg,
};
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::Version as ChannelVersion;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgUpdateClient};
use ibc::core::connection::types::msgs::{
    ConnectionMsg, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry,
};
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::Counterparty as ConnectionCounterParty;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath,
};
use ibc::core::host::ValidationContext;
use ibc::primitives::Signer;
use ibc_query::core::context::ProvableContext;

use crate::context::MockContext;
use crate::hosts::{HostClientState, TestBlock, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::{DefaultIbcStore, LightClientBuilder, LightClientState};

/// Implements relayer methods for a pair of hosts
/// Note that, all the implementations are in one direction, from A to B
/// For the methods in opposite direction, use `TypedIntegration::<B, A>` instead of TypedIntegration::<A, B>`
#[derive(Debug, Default)]
pub struct TypedIntegration<A, B>(PhantomData<A>, PhantomData<B>)
where
    A: TestHost,
    B: TestHost,
    HostClientState<A>: ClientStateValidation<DefaultIbcStore>,
    HostClientState<B>: ClientStateValidation<DefaultIbcStore>;

impl<A, B> TypedIntegration<A, B>
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

    pub fn sync_clock_on_a(ctx_a: &mut MockContext<A>, ctx_b: &MockContext<B>) {
        while ctx_b.latest_timestamp() > ctx_a.latest_timestamp() {
            ctx_a.advance_block();
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
        // this is required, as IBC doesn't allow client update
        // from future beyond max clock drift
        TypedIntegration::<A, B>::sync_clock_on_a(ctx_a, ctx_b);
        TypedIntegration::<A, B>::update_client_on_a(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer,
        );
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
            versions_on_a: ConnectionVersion::compatibles(),
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
            version: ConnectionVersion::compatibles()[0].clone(),
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
        let conn_id_on_a = TypedIntegration::<A, B>::connection_open_init_on_a(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        let conn_id_on_b = TypedIntegration::<A, B>::connection_open_try_on_b(
            ctx_b,
            router_b,
            ctx_a,
            conn_id_on_a.clone(),
            client_id_on_a.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::connection_open_ack_on_a(
            ctx_a,
            router_a,
            ctx_b,
            conn_id_on_a.clone(),
            conn_id_on_b.clone(),
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::connection_open_confirm_on_b(
            ctx_b,
            router_b,
            ctx_a,
            conn_id_on_b.clone(),
            conn_id_on_a.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer,
        );

        (conn_id_on_a, conn_id_on_b)
    }

    pub fn channel_open_init_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        conn_id_on_a: ConnectionId,
        port_id_on_a: PortId,
        port_id_on_b: PortId,
        signer: Signer,
    ) -> ChannelId {
        let msg_for_a = MsgEnvelope::Channel(ChannelMsg::OpenInit(MsgChannelOpenInit {
            port_id_on_a,
            connection_hops_on_a: [conn_id_on_a].to_vec(),
            port_id_on_b,
            ordering: Order::Unordered,
            signer,
            version_proposal: ChannelVersion::empty(),
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::OpenInitChannel(open_init_channel_event)) =
            ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        open_init_channel_event.chan_id_on_a().clone()
    }

    pub fn channel_open_try_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        conn_id_on_b: ConnectionId,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        signer: Signer,
    ) -> ChannelId {
        let proof_height_on_a = ctx_a.latest_height();

        let proof_chan_end_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proof_height_on_a,
                &ChannelEndPath::new(&port_id_on_a, &chan_id_on_a).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        #[allow(deprecated)]
        let msg_for_b = MsgEnvelope::Channel(ChannelMsg::OpenTry(MsgChannelOpenTry {
            port_id_on_b: PortId::transfer(),
            connection_hops_on_b: [conn_id_on_b].to_vec(),
            port_id_on_a: PortId::transfer(),
            chan_id_on_a,
            version_supported_on_a: ChannelVersion::empty(),
            proof_chan_end_on_a,
            proof_height_on_a,
            ordering: Order::Unordered,
            signer,

            version_proposal: ChannelVersion::empty(),
        }));

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::OpenTryChannel(open_try_channel_event)) =
            ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        open_try_channel_event.chan_id_on_b().clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn channel_open_ack_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        let proof_height_on_b = ctx_b.latest_height();

        let proof_chan_end_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proof_height_on_b,
                &ChannelEndPath::new(&port_id_on_b, &chan_id_on_b).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_a = MsgEnvelope::Channel(ChannelMsg::OpenAck(MsgChannelOpenAck {
            port_id_on_a,
            chan_id_on_a,
            chan_id_on_b,
            version_on_b: ChannelVersion::empty(),
            proof_chan_end_on_b,
            proof_height_on_b,
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::OpenAckChannel(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn channel_open_confirm_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        chan_id_on_a: ChannelId,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        let proof_height_on_a = ctx_a.latest_height();

        let proof_chan_end_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proof_height_on_a,
                &ChannelEndPath::new(&PortId::transfer(), &chan_id_on_a).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_b = MsgEnvelope::Channel(ChannelMsg::OpenConfirm(MsgChannelOpenConfirm {
            port_id_on_b,
            chan_id_on_b,
            proof_chan_end_on_a,
            proof_height_on_a,
            signer,
        }));

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::OpenConfirmChannel(_)) = ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn channel_close_init_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        signer: Signer,
    ) {
        let msg_for_a = MsgEnvelope::Channel(ChannelMsg::CloseInit(MsgChannelCloseInit {
            port_id_on_a,
            chan_id_on_a,
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::CloseInitChannel(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn channel_close_confirm_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        let proof_height_on_a = ctx_a.latest_height();

        let proof_chan_end_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proof_height_on_a,
                &ChannelEndPath::new(&PortId::transfer(), &chan_id_on_b).into(),
            )
            .expect("connection end exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_b = MsgEnvelope::Channel(ChannelMsg::CloseConfirm(MsgChannelCloseConfirm {
            port_id_on_b,
            chan_id_on_b,
            proof_chan_end_on_a,
            proof_height_on_a,
            signer,
        }));

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::CloseConfirmChannel(_)) =
            ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_channel_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        client_id_on_a: ClientId,
        conn_id_on_a: ConnectionId,
        port_id_on_a: PortId,
        client_id_on_b: ClientId,
        conn_id_on_b: ConnectionId,
        port_id_on_b: PortId,
        signer: Signer,
    ) -> (ChannelId, ChannelId) {
        let chan_id_on_a = TypedIntegration::<A, B>::channel_open_init_on_a(
            ctx_a,
            router_a,
            conn_id_on_a.clone(),
            port_id_on_a.clone(),
            port_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        let chan_id_on_b = TypedIntegration::<A, B>::channel_open_try_on_b(
            ctx_b,
            router_b,
            ctx_a,
            conn_id_on_b.clone(),
            chan_id_on_a.clone(),
            port_id_on_a.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::channel_open_ack_on_a(
            ctx_a,
            router_a,
            ctx_b,
            chan_id_on_a.clone(),
            port_id_on_a.clone(),
            chan_id_on_b.clone(),
            port_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::channel_open_confirm_on_b(
            ctx_b,
            router_b,
            ctx_a,
            chan_id_on_a.clone(),
            chan_id_on_b.clone(),
            port_id_on_b,
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer,
        );

        (chan_id_on_a, chan_id_on_b)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn close_channel_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        client_id_on_a: ClientId,
        chan_id_on_a: ChannelId,
        port_id_on_a: PortId,
        client_id_on_b: ClientId,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        TypedIntegration::<A, B>::channel_close_init_on_a(
            ctx_a,
            router_a,
            chan_id_on_a.clone(),
            port_id_on_a.clone(),
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b,
            signer.clone(),
        );

        TypedIntegration::<A, B>::channel_close_confirm_on_b(
            ctx_b,
            router_b,
            ctx_a,
            chan_id_on_b,
            port_id_on_b,
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer,
        );
    }

    pub fn packet_recv_on_b(
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        ctx_a: &MockContext<A>,
        packet: Packet,
        signer: Signer,
    ) -> Acknowledgement {
        let proof_height_on_a = ctx_a.latest_height();

        let proof_commitment_on_a = ctx_a
            .ibc_store()
            .get_proof(
                proof_height_on_a,
                &CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a)
                    .into(),
            )
            .expect("commitment proof exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_b = MsgEnvelope::Packet(PacketMsg::Recv(MsgRecvPacket {
            packet,
            proof_commitment_on_a,
            proof_height_on_a,
            signer,
        }));

        ctx_b.deliver(router_b, msg_for_b).expect("success");

        let Some(IbcEvent::WriteAcknowledgement(write_ack_event)) =
            ctx_b.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };

        write_ack_event.acknowledgement().clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn packet_ack_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        packet: Packet,
        acknowledgement: Acknowledgement,
        signer: Signer,
    ) {
        let proof_height_on_b = ctx_b.latest_height();

        let proof_acked_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proof_height_on_b,
                &AckPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a).into(),
            )
            .expect("acknowledgement proof exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_a = MsgEnvelope::Packet(PacketMsg::Ack(MsgAcknowledgement {
            packet,
            acknowledgement,
            proof_acked_on_b,
            proof_height_on_b,
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::AcknowledgePacket(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn packet_timeout_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        packet: Packet,
        signer: Signer,
    ) {
        let proof_height_on_b = ctx_b.latest_height();

        let proof_unreceived_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proof_height_on_b,
                &ReceiptPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a)
                    .into(),
            )
            .expect("non-membership receipt proof exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_a = MsgEnvelope::Packet(PacketMsg::Timeout(MsgTimeout {
            next_seq_recv_on_b: packet.seq_on_a,
            packet,
            proof_unreceived_on_b,
            proof_height_on_b,
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::TimeoutPacket(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    pub fn packet_timeout_on_close_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &MockContext<B>,
        packet: Packet,
        chan_id_on_b: ChannelId,
        port_id_on_b: PortId,
        signer: Signer,
    ) {
        let proof_height_on_b = ctx_b.latest_height();

        let proof_unreceived_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proof_height_on_b,
                &ReceiptPath::new(&port_id_on_b, &chan_id_on_b, packet.seq_on_a).into(),
            )
            .expect("non-membership receipt proof")
            .try_into()
            .expect("value merkle proof");

        let proof_close_on_b = ctx_b
            .ibc_store()
            .get_proof(
                proof_height_on_b,
                &ChannelEndPath::new(&port_id_on_b, &chan_id_on_b).into(),
            )
            .expect("channel end data exists")
            .try_into()
            .expect("value merkle proof");

        let msg_for_a = MsgEnvelope::Packet(PacketMsg::TimeoutOnClose(MsgTimeoutOnClose {
            next_seq_recv_on_b: packet.seq_on_a,
            packet,
            proof_unreceived_on_b,
            proof_close_on_b,
            proof_height_on_b,
            signer,
        }));

        ctx_a.deliver(router_a, msg_for_a).expect("success");

        let Some(IbcEvent::ChannelClosed(_)) = ctx_a.ibc_store().events.lock().last().cloned()
        else {
            panic!("unexpected event")
        };
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send_packet_on_a(
        ctx_a: &mut MockContext<A>,
        router_a: &mut MockRouter,
        ctx_b: &mut MockContext<B>,
        router_b: &mut MockRouter,
        packet: Packet,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
        signer: Signer,
    ) {
        // packet is passed from module

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b.clone(),
            signer.clone(),
        );

        let acknowledgement = TypedIntegration::<A, B>::packet_recv_on_b(
            ctx_b,
            router_b,
            ctx_a,
            packet.clone(),
            signer.clone(),
        );

        TypedIntegration::<A, B>::update_client_on_a_with_sync(
            ctx_a,
            router_a,
            ctx_b,
            client_id_on_a,
            signer.clone(),
        );

        TypedIntegration::<A, B>::packet_ack_on_a(
            ctx_a,
            router_a,
            ctx_b,
            packet,
            acknowledgement,
            signer.clone(),
        );

        TypedIntegration::<B, A>::update_client_on_a_with_sync(
            ctx_b,
            router_b,
            ctx_a,
            client_id_on_b,
            signer.clone(),
        );
    }
}
