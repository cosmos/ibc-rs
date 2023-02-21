use ibc_proto::google::protobuf::Any;

use super::{
    ics26_routing::{error::RouterError, msgs::MsgEnvelope},
    ExecutionContext, ValidationContext,
};

/// Entrypoint which only performs message validation
pub fn validate<Ctx>(ctx: &Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ValidationContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    ctx.validate(envelope)
}

/// Entrypoint which only performs message execution
pub fn execute<Ctx>(ctx: &mut Ctx, message: Any) -> Result<(), RouterError>
where
    Ctx: ExecutionContext,
{
    let envelope: MsgEnvelope = message.try_into()?;
    ctx.execute(envelope)
}

/// Entrypoint which performs both validation and message execution
pub fn dispatch(ctx: &mut impl ExecutionContext, msg: MsgEnvelope) -> Result<(), RouterError> {
    ctx.validate(msg.clone())?;
    ctx.execute(msg)
}

#[cfg(test)]
mod tests {
    use core::default::Default;
    use core::time::Duration;

    use test_log::test;

    use crate::applications::transfer::msgs::transfer::test_util::get_dummy_transfer_packet;
    use crate::applications::transfer::{
        context::test::deliver as ics20_deliver, msgs::transfer::test_util::get_dummy_msg_transfer,
        msgs::transfer::MsgTransfer, packet::PacketData, PrefixedCoin, MODULE_ID_STR,
    };
    use crate::core::context::Router;
    use crate::core::ics02_client::msgs::{
        create_client::MsgCreateClient, update_client::MsgUpdateClient,
        upgrade_client::MsgUpgradeClient, ClientMsg,
    };
    use crate::core::ics03_connection::connection::{
        ConnectionEnd, Counterparty as ConnCounterparty, State as ConnState,
    };
    use crate::core::ics03_connection::msgs::{
        conn_open_ack::MsgConnectionOpenAck, conn_open_init::MsgConnectionOpenInit,
        conn_open_try::MsgConnectionOpenTry, ConnectionMsg,
    };
    use crate::core::ics03_connection::version::Version as ConnVersion;
    use crate::core::ics04_channel::channel::ChannelEnd;
    use crate::core::ics04_channel::channel::Counterparty as ChannelCounterparty;
    use crate::core::ics04_channel::channel::Order as ChannelOrder;
    use crate::core::ics04_channel::channel::State as ChannelState;
    use crate::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_ack_with_packet;
    use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
    use crate::core::ics04_channel::msgs::chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm;
    use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
    use crate::core::ics04_channel::msgs::{
        chan_close_confirm::{
            test_util::get_dummy_raw_msg_chan_close_confirm, MsgChannelCloseConfirm,
        },
        chan_close_init::{test_util::get_dummy_raw_msg_chan_close_init, MsgChannelCloseInit},
        chan_open_ack::{test_util::get_dummy_raw_msg_chan_open_ack, MsgChannelOpenAck},
        chan_open_init::{test_util::get_dummy_raw_msg_chan_open_init, MsgChannelOpenInit},
        chan_open_try::{test_util::get_dummy_raw_msg_chan_open_try, MsgChannelOpenTry},
        recv_packet::{test_util::get_dummy_raw_msg_recv_packet, MsgRecvPacket},
        timeout_on_close::{test_util::get_dummy_raw_msg_timeout_on_close, MsgTimeoutOnClose},
        ChannelMsg, PacketMsg,
    };
    use crate::core::ics04_channel::timeout::TimeoutHeight;
    use crate::core::ics04_channel::Version as ChannelVersion;
    use crate::core::ics23_commitment::commitment::test_util::get_dummy_merkle_proof;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::core::ics24_host::path::CommitmentPath;
    use crate::core::ics26_routing::context::ModuleId;
    use crate::core::ics26_routing::error::RouterError;
    use crate::core::ics26_routing::msgs::MsgEnvelope;
    use crate::core::{dispatch, ValidationContext};
    use crate::events::IbcEvent;
    use crate::handler::HandlerOutputBuilder;
    use crate::mock::client_state::MockClientState;
    use crate::mock::consensus_state::MockConsensusState;
    use crate::mock::context::MockContext;
    use crate::mock::header::MockHeader;
    use crate::prelude::*;
    use crate::test_utils::{get_dummy_account_id, DummyTransferModule};
    use crate::timestamp::Timestamp;
    use crate::Height;

    #[test]
    /// These tests exercise two main paths: (1) the ability of the ICS26 routing module to dispatch
    /// messages to the correct module handler, and more importantly: (2) the ability of ICS handlers
    /// to work with the context and correctly store results.
    fn routing_module_and_keepers() {
        #[derive(Clone, Debug)]
        enum TestMsg {
            Ics26(MsgEnvelope),
            Ics20(MsgTransfer<PrefixedCoin>),
        }

        impl From<MsgEnvelope> for TestMsg {
            fn from(msg: MsgEnvelope) -> Self {
                Self::Ics26(msg)
            }
        }

        impl From<MsgTransfer<PrefixedCoin>> for TestMsg {
            fn from(msg: MsgTransfer<PrefixedCoin>) -> Self {
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

        let transfer_module_id: ModuleId = MODULE_ID_STR.parse().unwrap();

        // We reuse this same context across all tests. Nothing in particular needs parametrizing.
        let mut ctx = {
            let mut ctx = MockContext::default();
            let module = DummyTransferModule::new(ctx.ibc_store_share());
            ctx.add_route(transfer_module_id.clone(), module).unwrap();

            ctx
        };

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

        let msg_transfer = get_dummy_msg_transfer(Height::new(0, 35).unwrap().into(), None);
        let msg_transfer_two = get_dummy_msg_transfer(Height::new(0, 36).unwrap().into(), None);
        let msg_transfer_no_timeout = get_dummy_msg_transfer(TimeoutHeight::no_timeout(), None);
        let msg_transfer_no_timeout_or_timestamp = get_dummy_msg_transfer(
            TimeoutHeight::no_timeout(),
            Some(Timestamp::from_nanoseconds(0).unwrap()),
        );

        let mut msg_to_on_close =
            MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(36, 5)).unwrap();
        msg_to_on_close.packet.sequence = 2.into();
        msg_to_on_close.packet.timeout_height_on_b = msg_transfer_two.timeout_height_on_b;
        msg_to_on_close.packet.timeout_timestamp_on_b = msg_transfer_two.timeout_timestamp_on_b;

        let denom = msg_transfer_two.token.denom.clone();
        let packet_data = {
            let data = PacketData {
                token: PrefixedCoin {
                    denom,
                    amount: msg_transfer_two.token.amount,
                },
                sender: msg_transfer_two.sender.clone(),
                receiver: msg_transfer_two.receiver.clone(),
            };
            serde_json::to_vec(&data).expect("PacketData's infallible Serialize impl failed")
        };
        msg_to_on_close.packet.data = packet_data;

        let msg_recv_packet = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(35)).unwrap();
        let msg_ack_packet = MsgAcknowledgement::try_from(get_dummy_raw_msg_ack_with_packet(
            get_dummy_transfer_packet(msg_transfer.clone(), 1u64.into()).into(),
            35,
        ))
        .unwrap();

        // First, create a client..
        let res = dispatch(
            &mut ctx,
            MsgEnvelope::Client(ClientMsg::CreateClient(create_client_msg.clone())),
        );

        assert!(
            res.is_ok(),
            "ICS26 routing dispatch test 'client creation' failed for message {create_client_msg:?} with result: {res:?}",
        );

        ctx.scope_port_to_module(
            msg_chan_init.port_id_on_a.clone(),
            transfer_module_id.clone(),
        );

        // Figure out the ID of the client that was just created.
        let client_id_event = ctx.events.first();
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
                    header: MockHeader::new(update_client_height)
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
                    header: MockHeader::new(update_client_height).into(),
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
                    header: MockHeader::new(update_client_height_after_send)
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
                        &msg_ack_packet.packet.port_on_a,
                        &msg_ack_packet.packet.chan_on_a,
                        msg_ack_packet.packet.sequence,
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
                    header: MockHeader::new(update_client_height_after_second_send).into(),
                    signer: default_signer.clone(),
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
                msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(MsgUpgradeClient::new(
                    client_id.clone(),
                    MockClientState::new(MockHeader::new(upgrade_client_height)).into(),
                    MockConsensusState::new(MockHeader::new(upgrade_client_height)).into(),
                    get_dummy_merkle_proof(),
                    get_dummy_merkle_proof(),
                    default_signer.clone(),
                )))
                .into(),
                // Temporarily set to false due to the fact that the client
                // upgrade is not yet implemented
                want_pass: cfg!(feature = "upgrade_client"),
                state_check: None,
            },
            Test {
                name: "Client upgrade un-successful".to_string(),
                msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(MsgUpgradeClient::new(
                    client_id,
                    MockClientState::new(MockHeader::new(upgrade_client_height_second)).into(),
                    MockConsensusState::new(MockHeader::new(upgrade_client_height_second)).into(),
                    get_dummy_merkle_proof(),
                    get_dummy_merkle_proof(),
                    default_signer,
                )))
                .into(),
                want_pass: false,
                state_check: None,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = match test.msg.clone() {
                TestMsg::Ics26(msg) => dispatch(&mut ctx, msg).map(|_| ()),
                TestMsg::Ics20(msg) => {
                    let transfer_module = ctx.get_route_mut(&transfer_module_id).unwrap();
                    ics20_deliver(
                        transfer_module
                            .as_any_mut()
                            .downcast_mut::<DummyTransferModule>()
                            .unwrap(),
                        &mut HandlerOutputBuilder::new(),
                        msg,
                    )
                    .map(|_| ())
                    .map_err(|e| RouterError::ContextError(e.into()))
                }
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

    fn get_channel_events_ctx() -> MockContext {
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        let mut ctx = MockContext::default()
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
                ),
            );
        let module = DummyTransferModule::new(ctx.ibc_store_share());

        ctx.add_route(module_id.clone(), module).unwrap();

        // Note: messages will be using the default port
        ctx.scope_port_to_module(PortId::default(), module_id);

        ctx
    }

    #[test]
    fn test_chan_open_init_event() {
        let mut ctx = get_channel_events_ctx();

        let msg_chan_open_init =
            MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::OpenInit(msg_chan_open_init)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::OpenInitChannel(_)));
    }

    #[test]
    fn test_chan_open_try_event() {
        let mut ctx = get_channel_events_ctx();

        let msg_chan_open_try =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(1)).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::OpenTry(msg_chan_open_try)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::OpenTryChannel(_)));
    }

    #[test]
    fn test_chan_open_ack_event() {
        let mut ctx = get_channel_events_ctx().with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Init,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            ),
        );

        let msg_chan_open_ack =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(1)).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::OpenAck(msg_chan_open_ack)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::OpenAckChannel(_)));
    }

    #[test]
    fn test_chan_open_confirm_event() {
        let mut ctx = get_channel_events_ctx().with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::TryOpen,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            ),
        );

        let msg_chan_open_confirm =
            MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(1)).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::OpenConfirm(msg_chan_open_confirm)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::OpenConfirmChannel(_)));
    }

    #[test]
    fn test_chan_close_init_event() {
        let mut ctx = get_channel_events_ctx().with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Open,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            ),
        );

        let msg_chan_close_init =
            MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::CloseInit(msg_chan_close_init)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::CloseInitChannel(_)));
    }

    #[test]
    fn test_chan_close_confirm_event() {
        let mut ctx = get_channel_events_ctx().with_channel(
            PortId::default(),
            ChannelId::default(),
            ChannelEnd::new(
                ChannelState::Open,
                ChannelOrder::Unordered,
                ChannelCounterparty::new(PortId::default(), Some(ChannelId::default())),
                vec![ConnectionId::new(0)],
                ChannelVersion::default(),
            ),
        );

        let msg_chan_close_confirm =
            MsgChannelCloseConfirm::try_from(get_dummy_raw_msg_chan_close_confirm(1)).unwrap();

        dispatch(
            &mut ctx,
            MsgEnvelope::Channel(ChannelMsg::CloseConfirm(msg_chan_close_confirm)),
        )
        .unwrap();

        assert_eq!(ctx.events.len(), 1);

        let event = ctx.events.first().unwrap();

        assert!(matches!(event, IbcEvent::CloseConfirmChannel(_)));
    }
}
