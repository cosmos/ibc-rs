use ibc::apps::nft_transfer::module::{
    on_chan_open_init_execute, on_chan_open_init_validate, on_chan_open_try_execute,
    on_chan_open_try_validate,
};
use ibc::apps::nft_transfer::types::VERSION;
use ibc::core::channel::types::channel::{Counterparty, Order};
use ibc::core::channel::types::Version;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc::core::primitives::prelude::*;
use ibc_testkit::testapp::ibc::applications::nft_transfer::types::DummyNftTransferModule;

fn get_defaults() -> (
    DummyNftTransferModule,
    Order,
    Vec<ConnectionId>,
    PortId,
    ChannelId,
    Counterparty,
) {
    let order = Order::Unordered;
    let connection_hops = vec![ConnectionId::new(1)];
    let port_id = PortId::transfer();
    let channel_id = ChannelId::new(1);
    let counterparty = Counterparty::new(port_id.clone(), Some(channel_id.clone()));

    (
        DummyNftTransferModule,
        order,
        connection_hops,
        port_id,
        channel_id,
        counterparty,
    )
}

/// If the relayer passed "", indicating that it wants us to return the versions we support.
#[test]
fn test_on_chan_open_init_empty_version() {
    let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

    let in_version = Version::new("".to_string());

    let (_, out_version) = on_chan_open_init_execute(
        &mut ctx,
        order,
        &connection_hops,
        &port_id,
        &channel_id,
        &counterparty,
        &in_version,
    )
    .unwrap();

    assert_eq!(out_version, Version::new(VERSION.to_string()));
}

/// If the relayer passed in the only supported version (ics721-1), then return ics721-1
#[test]
fn test_on_chan_open_init_ics721_version() {
    let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

    let in_version = Version::new(VERSION.to_string());
    let (_, out_version) = on_chan_open_init_execute(
        &mut ctx,
        order,
        &connection_hops,
        &port_id,
        &channel_id,
        &counterparty,
        &in_version,
    )
    .unwrap();

    assert_eq!(out_version, Version::new(VERSION.to_string()));
}

/// If the relayer passed in an unsupported version, then fail
#[test]
fn test_on_chan_open_init_incorrect_version() {
    let (ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

    let in_version = Version::new("some-unsupported-version".to_string());
    let res = on_chan_open_init_validate(
        &ctx,
        order,
        &connection_hops,
        &port_id,
        &channel_id,
        &counterparty,
        &in_version,
    );

    assert!(res.is_err());
}

/// If the counterparty supports ics721, then return ics721
#[test]
fn test_on_chan_open_try_counterparty_correct_version() {
    let (mut ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

    let counterparty_version = Version::new(VERSION.to_string());

    let (_, out_version) = on_chan_open_try_execute(
        &mut ctx,
        order,
        &connection_hops,
        &port_id,
        &channel_id,
        &counterparty,
        &counterparty_version,
    )
    .unwrap();

    assert_eq!(out_version, Version::new(VERSION.to_string()));
}

/// If the counterparty doesn't support ics721, then fail
#[test]
fn test_on_chan_open_try_counterparty_incorrect_version() {
    let (ctx, order, connection_hops, port_id, channel_id, counterparty) = get_defaults();

    let counterparty_version = Version::new("some-unsupported-version".to_string());

    let res = on_chan_open_try_validate(
        &ctx,
        order,
        &connection_hops,
        &port_id,
        &channel_id,
        &counterparty,
        &counterparty_version,
    );

    assert!(res.is_err());
}
