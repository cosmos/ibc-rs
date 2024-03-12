use ibc::core::channel::types::proto::v1::MsgChannelCloseInit as RawMsgChannelCloseInit;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelCloseInit`, for testing purposes only!
pub fn dummy_raw_msg_chan_close_init() -> RawMsgChannelCloseInit {
    RawMsgChannelCloseInit {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::zero().to_string(),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::msgs::MsgChannelCloseInit;

    use super::*;

    #[test]
    fn parse_channel_close_init_msg() {
        struct Test {
            name: String,
            raw: RawMsgChannelCloseInit,
            want_pass: bool,
        }

        let default_raw_msg = dummy_raw_msg_chan_close_init();

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Correct port".to_string(),
                raw: RawMsgChannelCloseInit {
                    port_id: "p34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad port, name too short".to_string(),
                raw: RawMsgChannelCloseInit {
                    port_id: "p".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad port, name too long".to_string(),
                raw: RawMsgChannelCloseInit {
                    port_id: "abcdefsdfasdfasdfasdfasdfasdfadsfasdgafsgadfasdfasdfasdfsdfasdfaghijklmnopqrstuabcdefsdfasdfasdfasdfasdfasdfadsfasdgafsgadfasdfasdfasdfsdfasdfaghijklmnopqrstu".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Correct channel identifier".to_string(),
                raw: RawMsgChannelCloseInit {
                    channel_id: "channel-34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad channel, name too short".to_string(),
                raw: RawMsgChannelCloseInit {
                    channel_id: "chshort".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad channel, name too long".to_string(),
                raw: RawMsgChannelCloseInit {
                    channel_id: "channel-128391283791827398127398791283912837918273981273987912839".to_string(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ]
            .into_iter()
            .collect();

        for test in tests {
            let msg = MsgChannelCloseInit::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgChanCloseInit::try_from failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_chan_close_init();
        let msg = MsgChannelCloseInit::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelCloseInit::from(msg.clone());
        let msg_back = MsgChannelCloseInit::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
