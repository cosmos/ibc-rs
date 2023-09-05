use super::context::RouterError;
use super::ics02_client::handler::{create_client, update_client, upgrade_client};
use super::ics02_client::msgs::{ClientMsg, MsgUpdateOrMisbehaviour};
use super::ics03_connection::handler::{
    conn_open_ack, conn_open_confirm, conn_open_init, conn_open_try,
};
use super::ics03_connection::msgs::ConnectionMsg;
use super::ics04_channel::handler::acknowledgement::{
    acknowledgement_packet_execute, acknowledgement_packet_validate,
};
use super::ics04_channel::handler::chan_close_confirm::{
    chan_close_confirm_execute, chan_close_confirm_validate,
};
use super::ics04_channel::handler::chan_close_init::{
    chan_close_init_execute, chan_close_init_validate,
};
use super::ics04_channel::handler::chan_open_ack::{chan_open_ack_execute, chan_open_ack_validate};
use super::ics04_channel::handler::chan_open_confirm::{
    chan_open_confirm_execute, chan_open_confirm_validate,
};
use super::ics04_channel::handler::chan_open_init::{
    chan_open_init_execute, chan_open_init_validate,
};
use super::ics04_channel::handler::chan_open_try::{chan_open_try_execute, chan_open_try_validate};
use super::ics04_channel::handler::recv_packet::{recv_packet_execute, recv_packet_validate};
use super::ics04_channel::handler::timeout::{
    timeout_packet_execute, timeout_packet_validate, TimeoutMsgType,
};
use super::ics04_channel::msgs::{
    channel_msg_to_port_id, packet_msg_to_port_id, ChannelMsg, PacketMsg,
};
use super::msgs::MsgEnvelope;
use super::router::Router;
use super::{ExecutionContext, ValidationContext};

/// Entrypoint which performs both validation and message execution
pub fn dispatch(
    ctx: &mut impl ExecutionContext,
    router: &mut impl Router,
    msg: MsgEnvelope,
) -> Result<(), RouterError> {
    validate(ctx, router, msg.clone())?;
    execute(ctx, router, msg)
}

/// Entrypoint which only performs message validation
///
/// If a transaction contains `n` messages `m_1` ... `m_n`, then
/// they MUST be processed as follows:
///     validate(m_1), execute(m_1), ..., validate(m_n), execute(m_n)
/// That is, the state transition of message `i` must be applied before
/// message `i+1` is validated. This is equivalent to calling
/// `dispatch()` on each successively.
pub fn validate<Ctx>(
    ctx: &Ctx,
    router: &mut impl Router,
    msg: MsgEnvelope,
) -> Result<(), RouterError>
where
    Ctx: ValidationContext,
{
    match msg {
        MsgEnvelope::Client(msg) => match msg {
            ClientMsg::CreateClient(msg) => create_client::validate(ctx, msg),
            ClientMsg::UpdateClient(msg) => {
                update_client::validate(ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg))
            }
            ClientMsg::Misbehaviour(msg) => {
                update_client::validate(ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg))
            }
            ClientMsg::UpgradeClient(msg) => upgrade_client::validate(ctx, msg),
        }
        .map_err(RouterError::ContextError),
        MsgEnvelope::Connection(msg) => match msg {
            ConnectionMsg::OpenInit(msg) => conn_open_init::validate(ctx, msg),
            ConnectionMsg::OpenTry(msg) => conn_open_try::validate(ctx, msg),
            ConnectionMsg::OpenAck(msg) => conn_open_ack::validate(ctx, msg),
            ConnectionMsg::OpenConfirm(ref msg) => conn_open_confirm::validate(ctx, msg),
        }
        .map_err(RouterError::ContextError),
        MsgEnvelope::Channel(msg) => {
            let port_id = channel_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                ChannelMsg::OpenInit(msg) => chan_open_init_validate(ctx, module, msg),
                ChannelMsg::OpenTry(msg) => chan_open_try_validate(ctx, module, msg),
                ChannelMsg::OpenAck(msg) => chan_open_ack_validate(ctx, module, msg),
                ChannelMsg::OpenConfirm(msg) => chan_open_confirm_validate(ctx, module, msg),
                ChannelMsg::CloseInit(msg) => chan_close_init_validate(ctx, module, msg),
                ChannelMsg::CloseConfirm(msg) => chan_close_confirm_validate(ctx, module, msg),
            }
            .map_err(RouterError::ContextError)
        }
        MsgEnvelope::Packet(msg) => {
            let port_id = packet_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                PacketMsg::Recv(msg) => recv_packet_validate(ctx, msg),
                PacketMsg::Ack(msg) => acknowledgement_packet_validate(ctx, module, msg),
                PacketMsg::Timeout(msg) => {
                    timeout_packet_validate(ctx, module, TimeoutMsgType::Timeout(msg))
                }
                PacketMsg::TimeoutOnClose(msg) => {
                    timeout_packet_validate(ctx, module, TimeoutMsgType::TimeoutOnClose(msg))
                }
            }
            .map_err(RouterError::ContextError)
        }
    }
}

/// Entrypoint which only performs message execution
pub fn execute<Ctx>(
    ctx: &mut Ctx,
    router: &mut impl Router,
    msg: MsgEnvelope,
) -> Result<(), RouterError>
where
    Ctx: ExecutionContext,
{
    match msg {
        MsgEnvelope::Client(msg) => match msg {
            ClientMsg::CreateClient(msg) => create_client::execute(ctx, msg),
            ClientMsg::UpdateClient(msg) => {
                update_client::execute(ctx, MsgUpdateOrMisbehaviour::UpdateClient(msg))
            }
            ClientMsg::Misbehaviour(msg) => {
                update_client::execute(ctx, MsgUpdateOrMisbehaviour::Misbehaviour(msg))
            }
            ClientMsg::UpgradeClient(msg) => upgrade_client::execute(ctx, msg),
        }
        .map_err(RouterError::ContextError),
        MsgEnvelope::Connection(msg) => match msg {
            ConnectionMsg::OpenInit(msg) => conn_open_init::execute(ctx, msg),
            ConnectionMsg::OpenTry(msg) => conn_open_try::execute(ctx, msg),
            ConnectionMsg::OpenAck(msg) => conn_open_ack::execute(ctx, msg),
            ConnectionMsg::OpenConfirm(ref msg) => conn_open_confirm::execute(ctx, msg),
        }
        .map_err(RouterError::ContextError),
        MsgEnvelope::Channel(msg) => {
            let port_id = channel_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route_mut(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                ChannelMsg::OpenInit(msg) => chan_open_init_execute(ctx, module, msg),
                ChannelMsg::OpenTry(msg) => chan_open_try_execute(ctx, module, msg),
                ChannelMsg::OpenAck(msg) => chan_open_ack_execute(ctx, module, msg),
                ChannelMsg::OpenConfirm(msg) => chan_open_confirm_execute(ctx, module, msg),
                ChannelMsg::CloseInit(msg) => chan_close_init_execute(ctx, module, msg),
                ChannelMsg::CloseConfirm(msg) => chan_close_confirm_execute(ctx, module, msg),
            }
            .map_err(RouterError::ContextError)
        }
        MsgEnvelope::Packet(msg) => {
            let port_id = packet_msg_to_port_id(&msg);
            let module_id = router
                .lookup_module(port_id)
                .ok_or(RouterError::UnknownPort {
                    port_id: port_id.clone(),
                })?;
            let module = router
                .get_route_mut(&module_id)
                .ok_or(RouterError::ModuleNotFound)?;

            match msg {
                PacketMsg::Recv(msg) => recv_packet_execute(ctx, module, msg),
                PacketMsg::Ack(msg) => acknowledgement_packet_execute(ctx, module, msg),
                PacketMsg::Timeout(msg) => {
                    timeout_packet_execute(ctx, module, TimeoutMsgType::Timeout(msg))
                }
                PacketMsg::TimeoutOnClose(msg) => {
                    timeout_packet_execute(ctx, module, TimeoutMsgType::TimeoutOnClose(msg))
                }
            }
            .map_err(RouterError::ContextError)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::default::Default;
    use core::time::Duration;

    use test_log::test;

    use super::*;
    use crate::applications::transfer::error::TokenTransferError;
    use crate::applications::transfer::msgs::transfer::MsgTransfer;
    use crate::applications::transfer::{send_transfer, MODULE_ID_STR};
    use crate::core::dispatch;
    use crate::core::events::{IbcEvent, MessageEvent};
    use crate::core::ics02_client::msgs::create_client::MsgCreateClient;
    use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
    use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
    use crate::core::ics02_client::msgs::ClientMsg;
    use crate::core::ics03_connection::connection::{
        ConnectionEnd, Counterparty as ConnCounterparty, State as ConnState,
    };
    use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics03_connection::version::Version as ConnVersion;
    use crate::core::ics04_channel::channel::{
        ChannelEnd, Counterparty as ChannelCounterparty, Order as ChannelOrder,
        State as ChannelState,
    };
    use crate::core::ics04_channel::error::ChannelError;
    use crate::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_ack_with_packet;
    use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
    use crate::core::ics04_channel::msgs::chan_close_confirm::test_util::get_dummy_raw_msg_chan_close_confirm;
    use crate::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
    use crate::core::ics04_channel::msgs::chan_close_init::test_util::get_dummy_raw_msg_chan_close_init;
    use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
    use crate::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
    use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
    use crate::core::ics04_channel::msgs::chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm;
    use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
    use crate::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
    use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
    use crate::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::core::ics04_channel::msgs::recv_packet::test_util::get_dummy_raw_msg_recv_packet;
    use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
    use crate::core::ics04_channel::msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close;
    use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
    use crate::core::ics04_channel::msgs::{ChannelMsg, PacketMsg};
    use crate::core::ics04_channel::timeout::TimeoutHeight;
    use crate::core::ics04_channel::Version as ChannelVersion;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::core::ics24_host::path::CommitmentPath;
    use crate::core::msgs::MsgEnvelope;
    use crate::core::router::ModuleId;
    use crate::core::timestamp::Timestamp;
    use crate::mock::client_state::MockClientState;
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::mock::router::MockRouter;
    use crate::prelude::*;
    use crate::test_utils::{get_dummy_account_id, DummyTransferModule};
    use crate::Height;

    #[test]
    /// These tests exercise two main paths: (1) the ability of the ICS26 routing module to dispatch
    /// messages to the correct module handler, and more importantly: (2) the ability of ICS handlers
    /// to work with the context and correctly store results.
    fn routing_module_and_keepers() {
        #[derive(Clone, Debug)]
        enum TestMsg {
            Ics26(MsgEnvelope),
            Ics20(MsgTransfer),
        }

        impl From<MsgEnvelope> for TestMsg {
            fn from(msg: MsgEnvelope) -> Self {
                Self::Ics26(msg)
            }
        }

        impl From<MsgTransfer> for TestMsg {
            fn from(msg: MsgTransfer) -> Self {
                Self::Ics20(msg)
            }
        }

        type StateCheckFn = dyn FnOnce(&MockContext) -> bool;

        // Test parameters
        struct Test {
            name: String,
            msg: TestMsg,
            want_pass: bool,
            state_check: Option<Box<StateCheckFn>>,
        }
        let default_signer = get_dummy_account_id();
        let client_height = 5;
        let start_client_height = Height::new(0, client_height).unwrap();
        let update_client_height = Height::new(0, 34).unwrap();
        let update_client_height_after_send = Height::new(0, 35).unwrap();

        let update_client_height_after_second_send = Height::new(0, 36).unwrap();

        let upgrade_client_height = Height::new(1, 2).unwrap();

        let upgrade_client_height_second = Height::new(1, 1).unwrap();

        let transfer_module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());

        // We reuse this same context across all tests. Nothing in particular needs parametrizing.
        let mut ctx = MockContext::default();

        let mut router = MockRouter::default();
        router
            .add_route(transfer_module_id.clone(), DummyTransferModule::new())
            .unwrap();

        let create_client_msg = MsgCreateClient::new(
            MockClientState::new(MockHeader::new(start_client_height)).into(),
            MockConsensusState::new(MockHeader::new(start_client_height)).into(),
            default_signer.clone(),
        );

        //
        // Connection handshake messages.
        //
        let msg_conn_init = MsgConnectionOpenInit::new_dummy();

        let correct_msg_conn_try = MsgConnectionOpenTry::new_dummy(client_height, client_height);

        // The handler will fail to process this msg because the client height is too advanced.
        let incorrect_msg_conn_try =
            MsgConnectionOpenTry::new_dummy(client_height + 1, client_height + 1);

        let msg_conn_ack = MsgConnectionOpenAck::new_dummy(client_height, client_height);

        //
        // Channel handshake messages.
        //
        let msg_chan_init =
            MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        // The handler will fail to process this b/c the associated connection does not exist
        let mut incorrect_msg_chan_init = msg_chan_init.clone();
        incorrect_msg_chan_init.connection_hops_on_a = vec![ConnectionId::new(590)];

        let msg_chan_try =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(client_height)).unwrap();

        let msg_chan_ack =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(client_height)).unwrap();

        let msg_chan_close_init =
            MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

        let msg_chan_close_confirm =
            MsgChannelCloseConfirm::try_from(get_dummy_raw_msg_chan_close_confirm(client_height))
                .unwrap();

        let msg_transfer = MsgTransfer::new_dummy(Height::new(0, 35).unwrap().into(), None);
        let msg_transfer_two = MsgTransfer::new_dummy(Height::new(0, 36).unwrap().into(), None);
        let msg_transfer_no_timeout = MsgTransfer::new_dummy(TimeoutHeight::no_timeout(), None);
        let msg_transfer_no_timeout_or_timestamp = MsgTransfer::new_dummy(
            TimeoutHeight::no_timeout(),
            Some(Timestamp::from_nanoseconds(0).unwrap()),
        );

        let mut msg_to_on_close =
            MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(36, 5)).unwrap();
        msg_to_on_close.packet.seq_on_a = 2.into();
        msg_to_on_close.packet.timeout_height_on_b = msg_transfer_two.timeout_height_on_b;
        msg_to_on_close.packet.timeout_timestamp_on_b = msg_transfer_two.timeout_timestamp_on_b;

        let packet_data = serde_json::to_vec(&msg_transfer_two.packet_data)
            .expect("PacketData's infallible Serialize impl failed");

        msg_to_on_close.packet.data = packet_data;

        let msg_recv_packet = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(35)).unwrap();
        let msg_ack_packet = MsgAcknowledgement::try_from(get_dummy_raw_msg_ack_with_packet(
            msg_transfer.get_transfer_packet(1u64.into()).into(),
            35,
        ))
        .unwrap();

        // First, create a client..
        let res = dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Client(ClientMsg::CreateClient(create_client_msg.clone())),
        );

        assert!(
            res.is_ok(),
            "ICS26 routing dispatch test 'client creation' failed for message {create_client_msg:?} with result: {res:?}",
        );

        router.scope_port_to_module(msg_chan_init.port_id_on_a.clone(), transfer_module_id);

        // Figure out the ID of the client that was just created.
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Client)
        ));
        let client_id_event = ctx.events.get(1);
        assert!(
            client_id_event.is_some(),
            "There was no event generated for client creation!"
        );
        let client_id = match client_id_event.unwrap() {
            IbcEvent::CreateClient(create_client) => create_client.client_id().clone(),
            event => panic!("unexpected IBC event: {:?}", event),
        };

        let tests: Vec<Test> = vec![
            // Test some ICS2 client functionality.
            Test {
                name: "Client update successful".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
                    client_id: client_id.clone(),
                    client_message: MockHeader::new(update_client_height)
                        .with_timestamp(Timestamp::now())
                        .into(),
                    signer: default_signer.clone(),
                }))
                .into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Client update fails due to stale header".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
                    client_id: client_id.clone(),
                    client_message: MockHeader::new(update_client_height).into(),
                    signer: default_signer.clone(),
                }))
                .into(),
                want_pass: false,
                state_check: None,
            },
            Test {
                name: "Connection open init succeeds".to_string(),
                msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(
                    msg_conn_init.with_client_id(client_id.clone()),
                ))
                .into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Connection open try fails due to InvalidConsensusHeight (too high)"
                    .to_string(),
                msg: MsgEnvelope::Connection(ConnectionMsg::OpenTry(incorrect_msg_conn_try)).into(),
                want_pass: false,
                state_check: None,
            },
            Test {
                name: "Connection open try succeeds".to_string(),
                msg: MsgEnvelope::Connection(ConnectionMsg::OpenTry(
                    correct_msg_conn_try.with_client_id(client_id.clone()),
                ))
                .into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Connection open ack succeeds".to_string(),
                msg: MsgEnvelope::Connection(ConnectionMsg::OpenAck(msg_conn_ack)).into(),
                want_pass: true,
                state_check: None,
            },
            // ICS04
            Test {
                name: "Channel open init succeeds".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::OpenInit(msg_chan_init)).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Channel open init fail due to missing connection".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::OpenInit(incorrect_msg_chan_init)).into(),
                want_pass: false,
                state_check: None,
            },
            Test {
                name: "Channel open try succeeds".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::OpenTry(msg_chan_try)).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Channel open ack succeeds".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::OpenAck(msg_chan_ack)).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Packet send".to_string(),
                msg: msg_transfer.into(),
                want_pass: true,
                state_check: None,
            },
            // The client update is required in this test, because the proof associated with
            // msg_recv_packet has the same height as the packet TO height (see get_dummy_raw_msg_recv_packet)
            Test {
                name: "Client update successful #2".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
                    client_id: client_id.clone(),
                    client_message: MockHeader::new(update_client_height_after_send)
                        .with_timestamp(Timestamp::now())
                        .into(),
                    signer: default_signer.clone(),
                }))
                .into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Receive packet".to_string(),
                msg: MsgEnvelope::Packet(PacketMsg::Recv(msg_recv_packet.clone())).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Re-Receive packet".to_string(),
                msg: MsgEnvelope::Packet(PacketMsg::Recv(msg_recv_packet)).into(),
                want_pass: true,
                state_check: None,
            },
            // Ack packet
            Test {
                name: "Ack packet".to_string(),
                msg: MsgEnvelope::Packet(PacketMsg::Ack(msg_ack_packet.clone())).into(),
                want_pass: true,
                state_check: Some(Box::new(move |ctx| {
                    ctx.get_packet_commitment(&CommitmentPath::new(
                        &msg_ack_packet.packet.port_id_on_a,
                        &msg_ack_packet.packet.chan_id_on_a,
                        msg_ack_packet.packet.seq_on_a,
                    ))
                    .is_err()
                })),
            },
            Test {
                name: "Packet send".to_string(),
                msg: msg_transfer_two.into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Client update successful".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpdateClient(MsgUpdateClient {
                    client_id: client_id.clone(),
                    client_message: MockHeader::new(update_client_height_after_second_send).into(),
                    signer: default_signer,
                }))
                .into(),
                want_pass: true,
                state_check: None,
            },
            // Timeout packets
            Test {
                name: "Transfer message no timeout".to_string(),
                msg: msg_transfer_no_timeout.into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Transfer message no timeout nor timestamp".to_string(),
                msg: msg_transfer_no_timeout_or_timestamp.into(),
                want_pass: true,
                state_check: None,
            },
            //ICS04-close channel
            Test {
                name: "Channel close init succeeds".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::CloseInit(msg_chan_close_init)).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Channel close confirm fails cause channel is already closed".to_string(),
                msg: MsgEnvelope::Channel(ChannelMsg::CloseConfirm(msg_chan_close_confirm)).into(),
                want_pass: false,
                state_check: None,
            },
            //ICS04-to_on_close
            Test {
                name: "Timeout on close".to_string(),
                msg: MsgEnvelope::Packet(PacketMsg::TimeoutOnClose(msg_to_on_close)).into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Client upgrade successful".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(
                    MsgUpgradeClient::new_dummy(upgrade_client_height)
                        .with_client_id(client_id.clone()),
                ))
                .into(),
                want_pass: true,
                state_check: None,
            },
            Test {
                name: "Client upgrade un-successful".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(
                    MsgUpgradeClient::new_dummy(upgrade_client_height_second)
                        .with_client_id(client_id),
                ))
                .into(),
                want_pass: false,
                state_check: None,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = match test.msg.clone() {
                TestMsg::Ics26(msg) => dispatch(&mut ctx, &mut router, msg).map(|_| ()),
                TestMsg::Ics20(msg) => send_transfer(&mut ctx, &mut DummyTransferModule, msg, &())
                    .map_err(|e: TokenTransferError| ChannelError::AppModule {
                        description: e.to_string(),
                    })
                    .map_err(|e| RouterError::ContextError(e.into())),
            };

            assert_eq!(
                test.want_pass,
                res.is_ok(),
                "ICS26 routing dispatch test '{}' failed for message {:?}\nwith result: {:?}",
                test.name,
                test.msg,
                res
            );

            if let Some(state_check) = test.state_check {
                assert_eq!(
                    test.want_pass,
                    state_check(&ctx),
                    "ICS26 routing state check '{}' failed for message {:?}\nwith result: {:?}",
                    test.name,
                    test.msg,
                    res
                );
            }
        }
    }

    fn get_channel_events_ctx_router() -> (MockContext, MockRouter) {
        let module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());
        let ctx = MockContext::default()
            .with_client(&ClientId::default(), Height::new(0, 1).unwrap())
            .with_connection(
                ConnectionId::new(0),
                ConnectionEnd::new(
                    ConnState::Open,
                    ClientId::default(),
                    ConnCounterparty::new(
                        ClientId::default(),
                        Some(ConnectionId::new(0)),
                        CommitmentPrefix::default(),
                    ),
                    vec![ConnVersion::default()],
                    Duration::MAX,
                )
                .unwrap(),
            );
        let mut router = MockRouter::default();

        router
            .add_route(module_id.clone(), DummyTransferModule::new())
            .unwrap();

        // Note: messages will be using the default port
        router.scope_port_to_module(PortId::default(), module_id);

        (ctx, router)
    }

    #[test]
    fn test_chan_open_init_event() {
        let (mut ctx, mut router) = get_channel_events_ctx_router();

        let msg_chan_open_init =
            MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::OpenInit(msg_chan_open_init)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::OpenInitChannel(_)));
    }

    #[test]
    fn test_chan_open_try_event() {
        let (mut ctx, mut router) = get_channel_events_ctx_router();

        let msg_chan_open_try =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(1)).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::OpenTry(msg_chan_open_try)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::OpenTryChannel(_)));
    }

    #[test]
    fn test_chan_open_ack_event() {
        let (ctx, mut router) = get_channel_events_ctx_router();
        let mut ctx = ctx.with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Init,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            )
            .unwrap(),
        );

        let msg_chan_open_ack =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(1)).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::OpenAck(msg_chan_open_ack)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::OpenAckChannel(_)));
    }

    #[test]
    fn test_chan_open_confirm_event() {
        let (ctx, mut router) = get_channel_events_ctx_router();
        let mut ctx = ctx.with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::TryOpen,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            )
            .unwrap(),
        );

        let msg_chan_open_confirm =
            MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(1)).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::OpenConfirm(msg_chan_open_confirm)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::OpenConfirmChannel(_)));
    }

    #[test]
    fn test_chan_close_init_event() {
        let (ctx, mut router) = get_channel_events_ctx_router();
        let mut ctx = ctx.with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Open,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            )
            .unwrap(),
        );

        let msg_chan_close_init =
            MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::CloseInit(msg_chan_close_init)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::CloseInitChannel(_)));
    }

    #[test]
    fn test_chan_close_confirm_event() {
        let (ctx, mut router) = get_channel_events_ctx_router();
        let mut ctx = ctx.with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Open,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            )
            .unwrap(),
        );

        let msg_chan_close_confirm =
            MsgChannelCloseConfirm::try_from(get_dummy_raw_msg_chan_close_confirm(1)).unwrap();

        dispatch(
            &mut ctx,
            &mut router,
            MsgEnvelope::Channel(ChannelMsg::CloseConfirm(msg_chan_close_confirm)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::CloseConfirmChannel(_)));
    }
}
