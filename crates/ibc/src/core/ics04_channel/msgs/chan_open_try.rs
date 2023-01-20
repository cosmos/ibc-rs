use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::channel::Counterparty;
use crate::core::ics04_channel::channel::{Order, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::Version;
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::signer::Signer;
use crate::tx_msg::Msg;
use crate::{prelude::*, Height};

use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;
use ibc_proto::protobuf::Protobuf;

pub const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenTry";

///
/// Message definition for the second step in the channel open handshake (`ChanOpenTry` datagram).
/// Per our convention, this message is sent to chain B.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenTry {
    pub port_id_on_b: PortId,
    pub connection_hops_on_b: Vec<ConnectionId>,
    pub port_id_on_a: PortId,
    pub chan_id_on_a: ChannelId,
    pub version_supported_on_a: Version,
    pub proof_chan_end_on_a: CommitmentProofBytes,
    pub proof_height_on_a: Height,
    pub ordering: Order,
    pub signer: Signer,

    #[deprecated(since = "0.22.0")]
    /// Only kept here for proper conversion to/from the raw type
    pub previous_channel_id: String,
    #[deprecated(since = "0.22.0")]
    /// Only kept here for proper conversion to/from the raw type
    pub version_proposal: Version,
}

impl Msg for MsgChannelOpenTry {
    type Raw = RawMsgChannelOpenTry;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgChannelOpenTry> for MsgChannelOpenTry {}

impl TryFrom<RawMsgChannelOpenTry> for MsgChannelOpenTry {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenTry) -> Result<Self, Self::Error> {
        let chan_end_on_b: ChannelEnd = raw_msg
            .channel
            .ok_or(ChannelError::MissingChannel)?
            .try_into()?;
        #[allow(deprecated)]
        let msg = MsgChannelOpenTry {
            port_id_on_b: raw_msg.port_id.parse().map_err(ChannelError::Identifier)?,
            ordering: chan_end_on_b.ordering,
            previous_channel_id: raw_msg.previous_channel_id,
            connection_hops_on_b: chan_end_on_b.connection_hops,
            port_id_on_a: chan_end_on_b.remote.port_id,
            chan_id_on_a: chan_end_on_b
                .remote
                .channel_id
                .ok_or(ChannelError::InvalidCounterpartyChannelId)?,
            version_supported_on_a: raw_msg.counterparty_version.into(),
            proof_chan_end_on_a: raw_msg
                .proof_init
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.parse().map_err(ChannelError::Signer)?,
            version_proposal: chan_end_on_b.version,
        };

        Ok(msg)
    }
}

impl From<MsgChannelOpenTry> for RawMsgChannelOpenTry {
    fn from(domain_msg: MsgChannelOpenTry) -> Self {
        let chan_end_on_b = ChannelEnd::new(
            State::Init,
            domain_msg.ordering,
            Counterparty::new(domain_msg.port_id_on_a, Some(domain_msg.chan_id_on_a)),
            domain_msg.connection_hops_on_b,
            Version::empty(), // Excessive field to satisfy the type conversion
        );
        #[allow(deprecated)]
        RawMsgChannelOpenTry {
            port_id: domain_msg.port_id_on_b.to_string(),
            previous_channel_id: domain_msg.previous_channel_id,
            channel: Some(chan_end_on_b.into()),
            counterparty_version: domain_msg.version_supported_on_a.to_string(),
            proof_init: domain_msg.proof_chan_end_on_a.clone().into(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;

    use crate::core::ics04_channel::channel::test_util::get_dummy_raw_channel_end;
    use crate::core::ics24_host::identifier::{ChannelId, PortId};
    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};
    use ibc_proto::ibc::core::client::v1::Height;

    /// Returns a dummy `RawMsgChannelOpenTry`, for testing only!
    pub fn get_dummy_raw_msg_chan_open_try(proof_height: u64) -> RawMsgChannelOpenTry {
        #[allow(deprecated)]
        RawMsgChannelOpenTry {
            port_id: PortId::default().to_string(),
            previous_channel_id: ChannelId::default().to_string(),
            channel: Some(get_dummy_raw_channel_end(Some(0))),
            counterparty_version: "".to_string(),
            proof_init: get_dummy_proof(),
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
    use crate::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::prelude::*;

    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;
    use ibc_proto::ibc::core::client::v1::Height;
    use test_log::test;

    #[test]
    fn channel_open_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgChannelOpenTry,
            want_pass: bool,
        }

        let proof_height = 10;
        let default_raw_msg = get_dummy_raw_msg_chan_open_try(proof_height);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Correct port".to_string(),
                raw: RawMsgChannelOpenTry {
                    port_id: "p34".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad port, name too short".to_string(),
                raw: RawMsgChannelOpenTry {
                    port_id: "p".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad port, name too long".to_string(),
                raw: RawMsgChannelOpenTry {
                    port_id: "abcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfasdfaabcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfasdfa".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty counterparty version (valid choice)".to_string(),
                raw: RawMsgChannelOpenTry {
                    counterparty_version: " ".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Arbitrary counterparty version (valid choice)".to_string(),
                raw: RawMsgChannelOpenTry {
                    counterparty_version: "anyversion".to_string(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad proof height, height = 0".to_string(),
                raw: RawMsgChannelOpenTry {
                    proof_height: Some(Height {
                        revision_number: 0,
                        revision_height: 0,
                    }),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgChannelOpenTry {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof init (object proof)".to_string(),
                raw: RawMsgChannelOpenTry {
                    proof_init: Vec::new(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ]
            .into_iter()
            .collect();

        for test in tests {
            let res_msg = MsgChannelOpenTry::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "MsgChanOpenTry::try_from failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_chan_open_try(10);
        let msg = MsgChannelOpenTry::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelOpenTry::from(msg.clone());
        let msg_back = MsgChannelOpenTry::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
