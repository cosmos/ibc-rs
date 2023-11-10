use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State as ChannelState};
use ibc::core::ics04_channel::msgs::{ChannelMsg, MsgChannelCloseConfirm};
use ibc::core::ics04_channel::Version;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::timestamp::ZERO_DURATION;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::prelude::*;
use ibc_testkit::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use ibc_testkit::utils::dummies::core::channel::dummy_raw_msg_chan_close_confirm;
use ibc_testkit::utils::dummies::core::connection::dummy_raw_counterparty_conn;

#[test]
fn test_chan_close_confirm_validate() {
    let client_id = ClientId::new(mock_client_type(), 24).unwrap();
    let conn_id = ConnectionId::new(2);
    let default_context = MockContext::default();
    let client_consensus_state_height = default_context.host_height().unwrap();

    let conn_end = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::try_from(dummy_raw_counterparty_conn(Some(0))).unwrap(),
        get_compatible_versions(),
        ZERO_DURATION,
    )
    .unwrap();

    let msg_chan_close_confirm = MsgChannelCloseConfirm::try_from(
        dummy_raw_msg_chan_close_confirm(client_consensus_state_height.revision_height()),
    )
    .unwrap();

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg_chan_close_confirm.clone()));

    let chan_end = ChannelEnd::new(
        ChannelState::Open,
        Order::default(),
        Counterparty::new(
            msg_chan_close_confirm.port_id_on_b.clone(),
            Some(msg_chan_close_confirm.chan_id_on_b.clone()),
        ),
        vec![conn_id.clone()],
        Version::default(),
    )
    .unwrap();

    let context = default_context
        .with_client(&client_id, client_consensus_state_height)
        .with_connection(conn_id, conn_end)
        .with_channel(
            msg_chan_close_confirm.port_id_on_b.clone(),
            msg_chan_close_confirm.chan_id_on_b.clone(),
            chan_end,
        );

    let router = MockRouter::new_with_transfer();

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation expected to succeed (happy path). Error: {res:?}"
    );
}

#[test]
fn test_chan_close_confirm_execute() {
    let client_id = ClientId::new(mock_client_type(), 24).unwrap();
    let conn_id = ConnectionId::new(2);
    let default_context = MockContext::default();
    let client_consensus_state_height = default_context.host_height().unwrap();

    let conn_end = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::try_from(dummy_raw_counterparty_conn(Some(0))).unwrap(),
        get_compatible_versions(),
        ZERO_DURATION,
    )
    .unwrap();

    let msg_chan_close_confirm = MsgChannelCloseConfirm::try_from(
        dummy_raw_msg_chan_close_confirm(client_consensus_state_height.revision_height()),
    )
    .unwrap();

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg_chan_close_confirm.clone()));

    let chan_end = ChannelEnd::new(
        ChannelState::Open,
        Order::default(),
        Counterparty::new(
            msg_chan_close_confirm.port_id_on_b.clone(),
            Some(msg_chan_close_confirm.chan_id_on_b.clone()),
        ),
        vec![conn_id.clone()],
        Version::default(),
    )
    .unwrap();

    let mut context = default_context
        .with_client(&client_id, client_consensus_state_height)
        .with_connection(conn_id, conn_end)
        .with_channel(
            msg_chan_close_confirm.port_id_on_b.clone(),
            msg_chan_close_confirm.chan_id_on_b.clone(),
            chan_end,
        );

    let mut router = MockRouter::new_with_transfer();

    let res = execute(&mut context, &mut router, msg_envelope);

    assert!(res.is_ok(), "Execution success: happy path");

    assert_eq!(context.events.len(), 2);

    assert!(matches!(
        context.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));

    assert!(matches!(
        context.events[1],
        IbcEvent::CloseConfirmChannel(_)
    ));
}
