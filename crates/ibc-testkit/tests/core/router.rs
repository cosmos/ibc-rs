use ibc::applications::transfer::error::TokenTransferError;
use ibc::applications::transfer::msgs::transfer::MsgTransfer;
use ibc::applications::transfer::{send_transfer, BaseCoin};
use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use ibc::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use ibc::core::ics02_client::msgs::ClientMsg;
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::msgs::ConnectionMsg;
use ibc::core::ics04_channel::error::ChannelError;
use ibc::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_ack_with_packet;
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::chan_close_confirm::test_util::get_dummy_raw_msg_chan_close_confirm;
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::test_util::get_dummy_raw_msg_chan_close_init;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::test_util::get_dummy_raw_msg_recv_packet;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close;
use ibc::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use ibc::core::ics04_channel::msgs::{ChannelMsg, PacketMsg};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::core::ics24_host::path::CommitmentPath;
use ibc::core::timestamp::Timestamp;
use ibc::core::{dispatch, MsgEnvelope, RouterError, ValidationContext};
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc::mock::header::MockHeader;
use ibc::prelude::*;
use ibc::utils::dummy::get_dummy_account_id;
use ibc::Height;
use ibc_testkit::testapp::ibc::applications::transfer::configs::{
    extract_transfer_packet, MsgTransferConfig, PacketDataConfig,
};
use ibc_testkit::testapp::ibc::applications::transfer::types::DummyTransferModule;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use primitive_types::U256;
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
    let default_signer = get_dummy_account_id();
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
        MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(36, 5)).unwrap();
    msg_to_on_close.packet.seq_on_a = 2.into();
    msg_to_on_close.packet.timeout_height_on_b = msg_transfer_two.timeout_height_on_b;
    msg_to_on_close.packet.timeout_timestamp_on_b = msg_transfer_two.timeout_timestamp_on_b;

    let packet_data = serde_json::to_vec(&msg_transfer_two.packet_data)
        .expect("PacketData's infallible Serialize impl failed");

    msg_to_on_close.packet.data = packet_data;

    let msg_recv_packet = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(35)).unwrap();
    let msg_ack_packet = MsgAcknowledgement::try_from(get_dummy_raw_msg_ack_with_packet(
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
                    .with_current_timestamp()
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
            name: "Connection open try fails due to InvalidConsensusHeight (too high)".to_string(),
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
                MsgUpgradeClient::new_dummy(upgrade_client_height_second).with_client_id(client_id),
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
            TestMsg::Ics20(msg) => send_transfer(&mut ctx, &mut DummyTransferModule, msg)
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
