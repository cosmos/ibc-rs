use ibc::core::channel::types::proto::v1::MsgChannelCloseConfirm as RawMsgChannelCloseConfirm;
use ibc::core::client::types::proto::v1::Height;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use super::dummy_proof;
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelCloseConfirm`, for testing purposes only!
pub fn dummy_raw_msg_chan_close_confirm(proof_height: u64) -> RawMsgChannelCloseConfirm {
    RawMsgChannelCloseConfirm {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::zero().to_string(),
        proof_init: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
        counterparty_upgrade_sequence: 0,
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::msgs::MsgChannelCloseConfirm;

    use super::*;

    #[test]
    fn parse_channel_close_confirm_msg() {
        struct Test {
            name: String,
            raw: RawMsgChannelCloseConfirm,
            want_pass: bool,
        }

        let proof_height = 10;
        let default_raw_msg = dummy_raw_msg_chan_close_confirm(proof_height);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Correct port".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    port_id: "p34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad port, name too short".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    port_id: "p".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad port, name too long".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    port_id:
                        "abcdefghijklmnsdfasdfasdfasdfasdgafgadsfasdfasdfasdasfdasdfsadfopqrstuabcdefghijklmnsdfasdfasdfasdfasdgafgadsfasdfasdfasdasfdasdfsadfopqrstu"
                            .to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Correct channel identifier".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    channel_id: "channel-34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad channel, name too short".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    channel_id: "chshort".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad channel, name too long".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    channel_id:
                        "channel-128391283791827398127398791283912837918273981273987912839"
                            .to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height = 0".to_string(),
                raw: RawMsgChannelCloseConfirm {
                    proof_height: Some(Height {
                        revision_number: 0,
                        revision_height: 0,
                    }),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgChannelCloseConfirm::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgChanCloseConfirm::try_from raw failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_chan_close_confirm(19);
        let msg = MsgChannelCloseConfirm::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelCloseConfirm::from(msg.clone());
        let msg_back = MsgChannelCloseConfirm::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
