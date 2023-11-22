use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::{ChannelId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::{Msg, Signer};
use ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm as RawMsgChannelCloseConfirm;
use ibc_proto::Protobuf;

use crate::error::ChannelError;

pub const CHAN_CLOSE_CONFIRM_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelCloseConfirm";

///
/// Message definition for the second step in the channel close handshake (the `ChanCloseConfirm`
/// datagram).
/// Per our convention, this message is sent to chain B.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelCloseConfirm {
    pub port_id_on_b: PortId,
    pub chan_id_on_b: ChannelId,
    pub proof_chan_end_on_a: CommitmentProofBytes,
    pub proof_height_on_a: Height,
    pub signer: Signer,
}

impl Msg for MsgChannelCloseConfirm {
    type Raw = RawMsgChannelCloseConfirm;

    fn type_url(&self) -> String {
        CHAN_CLOSE_CONFIRM_TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgChannelCloseConfirm> for MsgChannelCloseConfirm {}

impl TryFrom<RawMsgChannelCloseConfirm> for MsgChannelCloseConfirm {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelCloseConfirm) -> Result<Self, Self::Error> {
        Ok(MsgChannelCloseConfirm {
            port_id_on_b: raw_msg.port_id.parse()?,
            chan_id_on_b: raw_msg.channel_id.parse()?,
            proof_chan_end_on_a: raw_msg
                .proof_init
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgChannelCloseConfirm> for RawMsgChannelCloseConfirm {
    fn from(domain_msg: MsgChannelCloseConfirm) -> Self {
        RawMsgChannelCloseConfirm {
            port_id: domain_msg.port_id_on_b.to_string(),
            channel_id: domain_msg.chan_id_on_b.to_string(),
            proof_init: domain_msg.proof_chan_end_on_a.clone().into(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ibc_primitives::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm as RawMsgChannelCloseConfirm;
    use ibc_proto::ibc::core::client::v1::Height;
    use ibc_testkit::utils::core::channel::dummy_raw_msg_chan_close_confirm;

    use crate::msgs::chan_close_confirm::MsgChannelCloseConfirm;

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
