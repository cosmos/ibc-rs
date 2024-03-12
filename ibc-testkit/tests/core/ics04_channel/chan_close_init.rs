use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State as ChannelState};
use ibc::core::channel::types::msgs::{ChannelMsg, MsgChannelCloseInit};
use ibc::core::channel::types::Version;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::*;
use ibc_testkit::fixtures::core::channel::dummy_raw_msg_chan_close_init;
use ibc_testkit::fixtures::core::connection::dummy_raw_counterparty_conn;
use ibc_testkit::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};

#[test]
fn test_chan_close_init_validate() {
    let client_id = mock_client_type().build_client_id(24);
    let conn_id = ConnectionId::new(2);

    let conn_end = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::try_from(dummy_raw_counterparty_conn(Some(0))).unwrap(),
        ConnectionVersion::compatibles(),
        ZERO_DURATION,
    )
    .unwrap();

    let msg_chan_close_init =
        MsgChannelCloseInit::try_from(dummy_raw_msg_chan_close_init()).unwrap();

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg_chan_close_init.clone()));

    let chan_end = ChannelEnd::new(
        ChannelState::Open,
        Order::Unordered,
        Counterparty::new(
            msg_chan_close_init.port_id_on_a.clone(),
            Some(msg_chan_close_init.chan_id_on_a.clone()),
        ),
        vec![conn_id.clone()],
        Version::empty(),
    )
    .unwrap();

    let context = {
        let default_context = MockContext::default();
        let client_consensus_state_height = default_context.host_height().unwrap();

        default_context
            .with_client_config(
                MockClientConfig::builder()
                    .client_id(client_id.clone())
                    .latest_height(client_consensus_state_height)
                    .build(),
            )
            .with_connection(conn_id, conn_end)
            .with_channel(
                msg_chan_close_init.port_id_on_a.clone(),
                msg_chan_close_init.chan_id_on_a.clone(),
                chan_end,
            )
    };

    let router = MockRouter::new_with_transfer();

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation expected to succeed (happy path). Error: {res:?}"
    );
}

#[test]
fn test_chan_close_init_execute() {
    let client_id = mock_client_type().build_client_id(24);
    let conn_id = ConnectionId::new(2);

    let conn_end = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::try_from(dummy_raw_counterparty_conn(Some(0))).unwrap(),
        ConnectionVersion::compatibles(),
        ZERO_DURATION,
    )
    .unwrap();

    let msg_chan_close_init =
        MsgChannelCloseInit::try_from(dummy_raw_msg_chan_close_init()).unwrap();

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg_chan_close_init.clone()));

    let chan_end = ChannelEnd::new(
        ChannelState::Open,
        Order::Unordered,
        Counterparty::new(
            msg_chan_close_init.port_id_on_a.clone(),
            Some(msg_chan_close_init.chan_id_on_a.clone()),
        ),
        vec![conn_id.clone()],
        Version::empty(),
    )
    .unwrap();

    let mut context = {
        let default_context = MockContext::default();
        let client_consensus_state_height = default_context.host_height().unwrap();

        default_context
            .with_client_config(
                MockClientConfig::builder()
                    .client_id(client_id.clone())
                    .latest_height(client_consensus_state_height)
                    .build(),
            )
            .with_connection(conn_id, conn_end)
            .with_channel(
                msg_chan_close_init.port_id_on_a.clone(),
                msg_chan_close_init.chan_id_on_a.clone(),
                chan_end,
            )
    };

    let mut router = MockRouter::new_with_transfer();

    let res = execute(&mut context, &mut router, msg_envelope);

    assert!(res.is_ok(), "Execution happy path");

    let ibc_events = context.get_events();

    assert_eq!(ibc_events.len(), 2);

    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));

    assert!(matches!(ibc_events[1], IbcEvent::CloseInitChannel(_)));
}
