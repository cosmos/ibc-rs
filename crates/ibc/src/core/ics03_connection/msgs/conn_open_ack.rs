use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics03_connection::error::Error;
use crate::core::ics03_connection::version::Version;
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::signer::Signer;
use crate::tx_msg::Msg;
use crate::Height;

pub const TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenAck";

/// Message definition `MsgConnectionOpenAck`  (i.e., `ConnOpenAck` datagram).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub client_state: Any,
    pub proof_connection_end: CommitmentProofBytes,
    pub proof_client_state: CommitmentProofBytes,
    pub proof_consensus_state: CommitmentProofBytes,
    pub proofs_height: Height,
    pub consensus_height: Height,
    pub version: Version,
    pub signer: Signer,
}

impl Msg for MsgConnectionOpenAck {
    type ValidationError = Error;
    type Raw = RawMsgConnectionOpenAck;

    fn route(&self) -> String {
        crate::keys::ROUTER_KEY.to_string()
    }

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgConnectionOpenAck> for MsgConnectionOpenAck {}

impl TryFrom<RawMsgConnectionOpenAck> for MsgConnectionOpenAck {
    type Error = Error;

    fn try_from(msg: RawMsgConnectionOpenAck) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: msg
                .connection_id
                .parse()
                .map_err(Error::invalid_identifier)?,
            counterparty_connection_id: msg
                .counterparty_connection_id
                .parse()
                .map_err(Error::invalid_identifier)?,
            client_state: msg.client_state.ok_or_else(Error::missing_client_state)?,
            version: msg.version.ok_or_else(Error::empty_versions)?.try_into()?,
            proof_connection_end: msg.proof_try.try_into().map_err(Error::invalid_proof)?,
            proof_client_state: msg.proof_client.try_into().map_err(Error::invalid_proof)?,
            proof_consensus_state: msg
                .proof_consensus
                .try_into()
                .map_err(Error::invalid_proof)?,
            proofs_height: msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or_else(Error::missing_proof_height)?,
            consensus_height: msg
                .consensus_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or_else(Error::missing_consensus_height)?,
            signer: msg.signer.parse().map_err(Error::signer)?,
        })
    }
}

impl From<MsgConnectionOpenAck> for RawMsgConnectionOpenAck {
    fn from(msg: MsgConnectionOpenAck) -> Self {
        RawMsgConnectionOpenAck {
            connection_id: msg.connection_id.as_str().to_string(),
            counterparty_connection_id: msg.counterparty_connection_id.as_str().to_string(),
            client_state: Some(msg.client_state),
            proof_height: Some(msg.proofs_height.into()),
            proof_try: msg.proof_connection_end.into(),
            proof_client: msg.proof_client_state.into(),
            proof_consensus: msg.proof_consensus_state.into(),
            consensus_height: Some(msg.consensus_height.into()),
            version: Some(msg.version.into()),
            signer: msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::core::ics02_client::height::Height;
    use crate::mock::client_state::MockClientState;
    use crate::mock::header::MockHeader;
    use crate::prelude::*;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;

    use crate::core::ics03_connection::version::Version;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};

    pub fn get_dummy_raw_msg_conn_open_ack(
        proof_height: u64,
        consensus_height: u64,
    ) -> RawMsgConnectionOpenAck {
        let client_state_height = Height::new(0, consensus_height).unwrap();
        RawMsgConnectionOpenAck {
            connection_id: ConnectionId::new(0).to_string(),
            counterparty_connection_id: ConnectionId::new(1).to_string(),
            proof_try: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: proof_height,
            }),
            proof_consensus: get_dummy_proof(),
            consensus_height: Some(RawHeight {
                revision_number: 0,
                revision_height: consensus_height,
            }),
            client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
            proof_client: get_dummy_proof(),
            version: Some(Version::default().into()),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::client::v1::Height;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;

    use crate::core::ics03_connection::msgs::conn_open_ack::test_util::get_dummy_raw_msg_conn_open_ack;
    use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;

    #[test]
    fn parse_connection_open_ack_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenAck,
            want_pass: bool,
        }

        let default_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_ack_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad connection id, non-alpha".to_string(),
                raw: RawMsgConnectionOpenAck {
                    connection_id: "con007".to_string(),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad version, missing version".to_string(),
                raw: RawMsgConnectionOpenAck {
                    version: None,
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    proof_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad consensus height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    consensus_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenAck::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenAck::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_conn_open_ack(5, 6);
        let msg = MsgConnectionOpenAck::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenAck::from(msg.clone());
        let msg_back = MsgConnectionOpenAck::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
