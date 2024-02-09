use ibc::apps::transfer::handler::send_transfer;
use ibc::apps::transfer::types::error::TokenTransferError;
use ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
use ibc::apps::transfer::types::{BaseCoin, U256};
use ibc::core::channel::types::error::ChannelError;
use ibc::core::channel::types::msgs::{
    ChannelMsg, MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeoutOnClose, PacketMsg,
};
use ibc::core::channel::types::timeout::TimeoutHeight;
use ibc::core::client::types::msgs::{ClientMsg, MsgCreateClient, MsgUpdateClient};
use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::ConnectionMsg;
use ibc::core::entrypoint::dispatch;
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::host::types::path::CommitmentPath;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc_testkit::fixtures::applications::transfer::{
    extract_transfer_packet, MsgTransferConfig, PacketDataConfig,
};
use ibc_testkit::fixtures::core::channel::{
    dummy_raw_msg_ack_with_packet, dummy_raw_msg_chan_close_confirm, dummy_raw_msg_chan_close_init,
    dummy_raw_msg_chan_open_ack, dummy_raw_msg_chan_open_init, dummy_raw_msg_chan_open_try,
    dummy_raw_msg_recv_packet, dummy_raw_msg_timeout_on_close,
};
use ibc_testkit::fixtures::core::client::dummy_msg_upgrade_client;
use ibc_testkit::fixtures::core::connection::{
    dummy_msg_conn_open_ack, dummy_msg_conn_open_init, dummy_msg_conn_open_init_with_client_id,
    dummy_msg_conn_open_try, msg_conn_open_try_with_client_id,
};
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::testapp::ibc::applications::transfer::types::DummyTransferModule;
use ibc_testkit::testapp::ibc::clients::mock::client_state::MockClientState;
use ibc_testkit::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use ibc_testkit::testapp::ibc::clients::mock::header::MockHeader;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use test_log::test;

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
    let default_signer = dummy_account_id();
    let client_height = 5;
    let start_client_height = Height::new(0, client_height).unwrap();
    let update_client_height = Height::new(0, 34).unwrap();
    let update_client_height_after_send = Height::new(0, 35).unwrap();

    let update_client_height_after_second_send = Height::new(0, 36).unwrap();

    let upgrade_client_height = Height::new(1, 2).unwrap();

    let upgrade_client_height_second = Height::new(1, 1).unwrap();

    // We reuse this same context across all tests. Nothing in particular needs parametrizing.
    let mut ctx = MockContext::default();

    let mut router = MockRouter::new_with_transfer();

    let create_client_msg = MsgCreateClient::new(
        MockClientState::new(MockHeader::new(start_client_height).with_current_timestamp()).into(),
        MockConsensusState::new(MockHeader::new(start_client_height).with_current_timestamp())
            .into(),
        default_signer.clone(),
    );

    //
    // Connection handshake messages.
    //
    let msg_conn_init = dummy_msg_conn_open_init();

    let correct_msg_conn_try = dummy_msg_conn_open_try(client_height, client_height);

    // The handler will fail to process this msg because the client height is too advanced.
    let incorrect_msg_conn_try = dummy_msg_conn_open_try(client_height + 1, client_height + 1);

    let msg_conn_ack = dummy_msg_conn_open_ack(client_height, client_height);

    //
    // Channel handshake messages.
    //
    let msg_chan_init = MsgChannelOpenInit::try_from(dummy_raw_msg_chan_open_init(None)).unwrap();

    // The handler will fail to process this b/c the associated connection does not exist
    let mut incorrect_msg_chan_init = msg_chan_init.clone();
    incorrect_msg_chan_init.connection_hops_on_a = vec![ConnectionId::new(590)];

    let msg_chan_try =
        MsgChannelOpenTry::try_from(dummy_raw_msg_chan_open_try(client_height)).unwrap();

    let msg_chan_ack =
        MsgChannelOpenAck::try_from(dummy_raw_msg_chan_open_ack(client_height)).unwrap();

    let msg_chan_close_init =
        MsgChannelCloseInit::try_from(dummy_raw_msg_chan_close_init()).unwrap();

    let msg_chan_close_confirm =
        MsgChannelCloseConfirm::try_from(dummy_raw_msg_chan_close_confirm(client_height)).unwrap();

    let packet_data = PacketDataConfig::builder()
        .token(
            BaseCoin {
                denom: "uatom".parse().expect("parse denom"),
                amount: U256::from(10).into(),
            }
            .into(),
        )
        .build();

    let msg_transfer = MsgTransferConfig::builder()
        .packet_data(packet_data.clone())
        .timeout_height_on_b(TimeoutHeight::At(Height::new(0, 35).unwrap()))
        .build();

    let msg_transfer_two = MsgTransferConfig::builder()
        .packet_data(packet_data.clone())
        .timeout_height_on_b(TimeoutHeight::At(Height::new(0, 36).unwrap()))
        .build();

    let msg_transfer_no_timeout = MsgTransferConfig::builder()
        .packet_data(packet_data.clone())
        .build();

    let msg_transfer_no_timeout_or_timestamp = MsgTransferConfig::builder()
        .packet_data(packet_data.clone())
        .timeout_timestamp_on_b(Timestamp::from_nanoseconds(0).unwrap())
        .build();

    let mut msg_to_on_close =
        MsgTimeoutOnClose::try_from(dummy_raw_msg_timeout_on_close(36, 5)).unwrap();
    msg_to_on_close.packet.seq_on_a = 2.into();
    msg_to_on_close.packet.timeout_height_on_b = msg_transfer_two.timeout_height_on_b;
    msg_to_on_close.packet.timeout_timestamp_on_b = msg_transfer_two.timeout_timestamp_on_b;

    let packet_data = serde_json::to_vec(&msg_transfer_two.packet_data)
        .expect("PacketData's infallible Serialize impl failed");

    msg_to_on_close.packet.data = packet_data;

    let msg_recv_packet = MsgRecvPacket::try_from(dummy_raw_msg_recv_packet(35)).unwrap();
    let msg_ack_packet = MsgAcknowledgement::try_from(dummy_raw_msg_ack_with_packet(
        extract_transfer_packet(&msg_transfer, 1u64.into()).into(),
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

    let ibc_events = ctx.get_events();

    // Figure out the ID of the client that was just created.
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Client)
    ));
    let client_id_event = ibc_events.get(1);
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
                    .with_current_timestamp()
                    .into(),
                signer: default_signer.clone(),
            }))
            .into(),
            want_pass: true,
            state_check: None,
        },
        Test {
            name: "Connection open init succeeds".to_string(),
            msg: MsgEnvelope::Connection(ConnectionMsg::OpenInit(
                dummy_msg_conn_open_init_with_client_id(msg_conn_init, client_id.clone()),
            ))
            .into(),
            want_pass: true,
            state_check: None,
        },
        Test {
            name: "Connection open try fails due to InvalidConsensusHeight (too high)".to_string(),
            msg: MsgEnvelope::Connection(ConnectionMsg::OpenTry(incorrect_msg_conn_try)).into(),
            want_pass: false,
            state_check: None,
        },
        Test {
            name: "Connection open try succeeds".to_string(),
            msg: MsgEnvelope::Connection(ConnectionMsg::OpenTry(msg_conn_open_try_with_client_id(
                correct_msg_conn_try,
                client_id.clone(),
            )))
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
                    .with_current_timestamp()
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
                client_message: MockHeader::new(update_client_height_after_second_send)
                    .with_current_timestamp()
                    .into(),
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
            msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(dummy_msg_upgrade_client(
                client_id.clone(),
                upgrade_client_height,
            )))
            .into(),
            want_pass: true,
            state_check: None,
        },
        Test {
            name: "Client upgrade un-successful".to_string(),
            msg: MsgEnvelope::Client(ClientMsg::UpgradeClient(dummy_msg_upgrade_client(
                client_id,
                upgrade_client_height_second,
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
            TestMsg::Ics26(msg) => dispatch(&mut ctx, &mut router, msg).map(|_| ()),
            TestMsg::Ics20(msg) => send_transfer(&mut ctx, &mut DummyTransferModule, msg)
                .map_err(|e: TokenTransferError| ChannelError::AppModule {
                    description: e.to_string(),
                })
                .map_err(ContextError::from),
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
