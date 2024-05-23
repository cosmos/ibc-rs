use ibc::apps::transfer::module::{
    on_chan_open_init_execute, on_chan_open_init_validate, on_chan_open_try_execute,
    on_chan_open_try_validate,
};
use ibc::apps::transfer::types::VERSION;
use ibc::core::channel::types::channel::{Counterparty, Order};
use ibc::core::channel::types::Version;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc::core::primitives::prelude::*;
use ibc::cosmos_host::utils::cosmos_adr028_escrow_address;
use ibc_testkit::testapp::ibc::applications::transfer::types::DummyTransferModule;
use subtle_encoding::bech32;

fn get_defaults() -> (
    DummyTransferModule,
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
        DummyTransferModule,
        order,
        connection_hops,
        port_id,
        channel_id,
        counterparty,
    )
}

#[test]
fn test_cosmos_escrow_address() {
    fn assert_eq_escrow_address(port_id: &str, channel_id: &str, address: &str) {
        let port_id = port_id.parse().unwrap();
        let channel_id = channel_id.parse().unwrap();
        let gen_address = {
            let addr = cosmos_adr028_escrow_address(&port_id, &channel_id);
            bech32::encode("cosmos", addr)
        };
        assert_eq!(gen_address, address.to_owned())
    }

    // addresses obtained using `gaiad query ibc-transfer escrow-address [port-id] [channel-id]`
    assert_eq_escrow_address(
        "transfer",
        "channel-141",
        "cosmos1x54ltnyg88k0ejmk8ytwrhd3ltm84xehrnlslf",
    );
    assert_eq_escrow_address(
        "transfer",
        "channel-207",
        "cosmos1ju6tlfclulxumtt2kglvnxduj5d93a64r5czge",
    );
    assert_eq_escrow_address(
        "transfer",
        "channel-187",
        "cosmos177x69sver58mcfs74x6dg0tv6ls4s3xmmcaw53",
    );
}

/// If the relayer passed "", indicating that it wants us to return the versions we support.
/// We currently only support ics20
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

/// If the relayer passed in the only supported version (ics20), then return ics20
#[test]
fn test_on_chan_open_init_ics20_version() {
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

/// If the counterparty supports ics20, then return ics20
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

/// If the counterparty doesn't support ics20, then fail
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
