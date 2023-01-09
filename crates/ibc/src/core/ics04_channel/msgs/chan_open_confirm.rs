use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::signer::Signer;
use crate::tx_msg::Msg;
use crate::{prelude::*, Height};

use ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;
use ibc_proto::protobuf::Protobuf;

pub const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenConfirm";

///
/// Message definition for the fourth step in the channel open handshake (`ChanOpenConfirm`
/// datagram).
/// Per our convention, this message is sent to chain B.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenConfirm {
    pub port_id_on_b: PortId,
    pub chan_id_on_b: ChannelId,
    pub proof_chan_end_on_a: CommitmentProofBytes,
    pub proof_height_on_a: Height,
    pub signer: Signer,
}

impl Msg for MsgChannelOpenConfirm {
    type Raw = RawMsgChannelOpenConfirm;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgChannelOpenConfirm> for MsgChannelOpenConfirm {}

impl TryFrom<RawMsgChannelOpenConfirm> for MsgChannelOpenConfirm {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenConfirm) -> Result<Self, Self::Error> {
        Ok(MsgChannelOpenConfirm {
            port_id_on_b: raw_msg.port_id.parse().map_err(ChannelError::Identifier)?,
            chan_id_on_b: raw_msg
                .channel_id
                .parse()
                .map_err(ChannelError::Identifier)?,
            proof_chan_end_on_a: raw_msg
                .proof_ack
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.parse().map_err(ChannelError::Signer)?,
        })
    }
}

impl From<MsgChannelOpenConfirm> for RawMsgChannelOpenConfirm {
    fn from(domain_msg: MsgChannelOpenConfirm) -> Self {
        RawMsgChannelOpenConfirm {
            port_id: domain_msg.port_id_on_b.to_string(),
            channel_id: domain_msg.chan_id_on_b.to_string(),
            proof_ack: domain_msg.proof_chan_end_on_a.into(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;

    use crate::core::ics24_host::identifier::{ChannelId, PortId};
    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};
    use ibc_proto::ibc::core::client::v1::Height;

    /// Returns a dummy `RawMsgChannelOpenConfirm`, for testing only!
    pub fn get_dummy_raw_msg_chan_open_confirm(proof_height: u64) -> RawMsgChannelOpenConfirm {
        RawMsgChannelOpenConfirm {
            port_id: PortId::default().to_string(),
            channel_id: ChannelId::default().to_string(),
            proof_ack: get_dummy_proof(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: proof_height,
            }),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;
    use test_log::test;

    use crate::core::ics04_channel::msgs::chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm;
    use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;

    use ibc_proto::ibc::core::client::v1::Height;

    #[test]
    fn parse_channel_open_confirm_msg() {
        struct Test {
            name: String,
            raw: RawMsgChannelOpenConfirm,
            want_pass: bool,
        }

        let proof_height = 78;
        let default_raw_msg = get_dummy_raw_msg_chan_open_confirm(proof_height);

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
        let raw = get_dummy_raw_msg_chan_open_confirm(19);
        let msg = MsgChannelOpenConfirm::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelOpenConfirm::from(msg.clone());
        let msg_back = MsgChannelOpenConfirm::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
