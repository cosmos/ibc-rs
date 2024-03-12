use ibc::core::channel::types::proto::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;
use ibc::core::client::types::proto::v1::Height;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use super::dummy_proof;
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenConfirm`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_confirm(proof_height: u64) -> RawMsgChannelOpenConfirm {
    RawMsgChannelOpenConfirm {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::zero().to_string(),
        proof_ack: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::msgs::MsgChannelOpenConfirm;

    use super::*;

    #[test]
    fn parse_channel_open_confirm_msg() {
        struct Test {
            name: String,
            raw: RawMsgChannelOpenConfirm,
            want_pass: bool,
        }

        let proof_height = 78;
        let default_raw_msg = dummy_raw_msg_chan_open_confirm(proof_height);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Correct port".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    port_id: "p34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad port, name too short".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    port_id: "p".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad port, name too long".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    port_id: "abcdesdfasdsdffasdfasdfasfasdgasdfgasdfasdfasdfasdfasdfasdffghijklmnopqrstuabcdesdfasdsdffasdfasdfasfasdgasdfgasdfasdfasdfasdfasdfasdffghijklmnopqrstu".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Correct channel identifier".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    channel_id: "channel-34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad channel, name too short".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    channel_id: "chshort".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad channel, name too long".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    channel_id: "channel-128391283791827398127398791283912837918273981273987912839".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height = 0".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    proof_height: Some(Height {
                        revision_number: 0,
                        revision_height: 0,
                    }),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing object proof".to_string(),
                raw: RawMsgChannelOpenConfirm {
                    proof_ack: Vec::new(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ]
            .into_iter()
            .collect();

        for test in tests {
            let res_msg = MsgChannelOpenConfirm::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "MsgChanOpenConfirm::try_from failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_chan_open_confirm(19);
        let msg = MsgChannelOpenConfirm::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelOpenConfirm::from(msg.clone());
        let msg_back = MsgChannelOpenConfirm::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
