mod acknowledgement;
mod chan_close_confirm;
mod chan_close_init;
mod chan_open_ack;
mod chan_open_confirm;
mod chan_open_init;
mod chan_open_try;
mod packet;
mod recv_packet;
mod timeout;
mod timeout_on_close;

pub use acknowledgement::*;
pub use chan_close_confirm::*;
pub use chan_close_init::*;
pub use chan_open_ack::*;
pub use chan_open_confirm::*;
pub use chan_open_init::*;
pub use chan_open_try::*;
use ibc::core::channel::types::proto::v1::{
    Channel as RawChannel, Counterparty as RawCounterparty,
};
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc::primitives::prelude::*;
pub use packet::*;
pub use recv_packet::*;
pub use timeout::*;
pub use timeout_on_close::*;

/// Returns a dummy `RawCounterparty`, for testing purposes only!
/// Can be optionally parametrized with a specific channel identifier.
pub fn dummy_raw_counterparty_chan(channel_id: String) -> RawCounterparty {
    RawCounterparty {
        port_id: PortId::transfer().to_string(),
        channel_id,
    }
}

/// Returns a dummy `RawChannel`, for testing purposes only!
pub fn dummy_raw_channel_end(state: i32, channel_id: Option<u64>) -> RawChannel {
    let channel_id = match channel_id {
        Some(id) => ChannelId::new(id).to_string(),
        None => "".to_string(),
    };
    RawChannel {
        state,
        ordering: 2,
        counterparty: Some(dummy_raw_counterparty_chan(channel_id)),
        connection_hops: vec![ConnectionId::zero().to_string()],
        version: "".to_string(), // The version is not validated.
        upgrade_sequence: 0,
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use ibc::core::channel::types::channel::ChannelEnd;

    use super::*;
    #[test]
    fn channel_end_try_from_raw() {
        let raw_channel_end = dummy_raw_channel_end(2, Some(0));

        let empty_raw_channel_end = RawChannel {
            counterparty: None,
            ..raw_channel_end.clone()
        };

        struct Test {
            name: String,
            params: RawChannel,
            want_pass: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                name: "Raw channel end with missing counterparty".to_string(),
                params: empty_raw_channel_end,
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect state".to_string(),
                params: RawChannel {
                    state: -1,
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect ordering".to_string(),
                params: RawChannel {
                    ordering: -1,
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect connection id in connection hops".to_string(),
                params: RawChannel {
                    connection_hops: vec!["connection*".to_string()].into_iter().collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect connection id (has blank space)".to_string(),
                params: RawChannel {
                    connection_hops: vec!["con nection".to_string()].into_iter().collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with two correct connection ids in connection hops"
                    .to_string(),
                params: RawChannel {
                    connection_hops: vec!["connection-1".to_string(), "connection-2".to_string()]
                        .into_iter()
                        .collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Raw channel end with correct params".to_string(),
                params: raw_channel_end,
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let ce_result = ChannelEnd::try_from(p);

            assert_eq!(
                test.want_pass,
                ce_result.is_ok(),
                "ChannelEnd::try_from() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                ce_result.err(),
            );
        }
    }

    #[test]
    fn parse_channel_ordering_type() {
        use ibc::core::channel::types::channel::Order;

        struct Test {
            ordering: &'static str,
            want_res: Order,
            want_err: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                ordering: "UNINITIALIZED",
                want_res: Order::None,
                want_err: false,
            },
            Test {
                ordering: "UNORDERED",
                want_res: Order::Unordered,
                want_err: false,
            },
            Test {
                ordering: "ORDERED",
                want_res: Order::Ordered,
                want_err: false,
            },
            Test {
                ordering: "UNKNOWN_ORDER",
                want_res: Order::None,
                want_err: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            match Order::from_str(test.ordering) {
                Ok(res) => {
                    assert!(!test.want_err);
                    assert_eq!(test.want_res, res);
                }
                Err(_) => assert!(test.want_err, "parse failed"),
            }
        }
    }
}
