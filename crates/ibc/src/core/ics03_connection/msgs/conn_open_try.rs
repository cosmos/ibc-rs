use crate::prelude::*;

use core::{
    convert::{TryFrom, TryInto},
    time::Duration,
};

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics03_connection::connection::Counterparty;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::version::Version;
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::Signer;
use crate::tx_msg::Msg;
use crate::Height;

pub const TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenTry";

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgConnectionOpenTry {
    /// ClientId on B that the connection is being opened for
    pub client_id_on_b: ClientId,
    /// ClientState of client tracking chain B on chain A
    pub client_state_of_b_on_a: Any,
    /// ClientId, ConnectionId and prefix of chain A
    pub counterparty: Counterparty,
    /// Versions supported by chain A
    pub versions_on_a: Vec<Version>,
    /// proof of ConnectionEnd stored on Chain A during ConnOpenInit
    pub proof_conn_end_on_a: CommitmentProofBytes,
    /// proof that chain A has stored ClientState of chain B on its client
    pub proof_client_state_of_b_on_a: CommitmentProofBytes,
    /// proof that chain A has stored ConsensusState of chain B on its client
    pub proof_consensus_state_of_b_on_a: CommitmentProofBytes,
    /// Height at which all proofs in this message were taken
    pub proofs_height_on_a: Height,
    /// height of latest header of chain A that updated the client on chain B
    pub consensus_height_of_b_on_a: Height,
    pub delay_period: Duration,
    pub signer: Signer,

    #[deprecated(since = "0.22.0")]
    /// Only kept here for proper conversion to/from the raw type
    previous_connection_id: String,
}

impl Msg for MsgConnectionOpenTry {
    type Raw = RawMsgConnectionOpenTry;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgConnectionOpenTry> for MsgConnectionOpenTry {}

impl TryFrom<RawMsgConnectionOpenTry> for MsgConnectionOpenTry {
    type Error = ConnectionError;

    fn try_from(msg: RawMsgConnectionOpenTry) -> Result<Self, Self::Error> {
        let counterparty_versions = msg
            .counterparty_versions
            .into_iter()
            .map(Version::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        if counterparty_versions.is_empty() {
            return Err(ConnectionError::EmptyVersions);
        }

        // We set the deprecated `previous_connection_id` field so that we can
        // properly convert `MsgConnectionOpenTry` into its raw form
        #[allow(deprecated)]
        Ok(Self {
            previous_connection_id: msg.previous_connection_id,
            client_id_on_b: msg
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            client_state_of_b_on_a: msg
                .client_state
                .ok_or(ConnectionError::MissingClientState)?,
            counterparty: msg
                .counterparty
                .ok_or(ConnectionError::MissingCounterparty)?
                .try_into()?,
            versions_on_a: counterparty_versions,
            proof_conn_end_on_a: msg
                .proof_init
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_client_state_of_b_on_a: msg
                .proof_client
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_consensus_state_of_b_on_a: msg
                .proof_consensus
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proofs_height_on_a: msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingProofHeight)?,
            consensus_height_of_b_on_a: msg
                .consensus_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingConsensusHeight)?,
            delay_period: Duration::from_nanos(msg.delay_period),
            signer: msg.signer.parse().map_err(ConnectionError::Signer)?,
        })
    }
}

impl From<MsgConnectionOpenTry> for RawMsgConnectionOpenTry {
    fn from(msg: MsgConnectionOpenTry) -> Self {
        #[allow(deprecated)]
        RawMsgConnectionOpenTry {
            client_id: msg.client_id_on_b.as_str().to_string(),
            previous_connection_id: msg.previous_connection_id,
            client_state: Some(msg.client_state_of_b_on_a),
            counterparty: Some(msg.counterparty.into()),
            delay_period: msg.delay_period.as_nanos() as u64,
            counterparty_versions: msg.versions_on_a.iter().map(|v| v.clone().into()).collect(),
            proof_height: Some(msg.proofs_height_on_a.into()),
            proof_init: msg.proof_conn_end_on_a.into(),
            proof_client: msg.proof_client_state_of_b_on_a.into(),
            proof_consensus: msg.proof_consensus_state_of_b_on_a.into(),
            consensus_height: Some(msg.consensus_height_of_b_on_a.into()),
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
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;

    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};

    /// Testing-specific helper methods.
    impl MsgConnectionOpenTry {
        /// Returns a new `MsgConnectionOpenTry` with dummy values.
        pub fn new_dummy(proof_height: u64, consensus_height: u64) -> Self {
            MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
                proof_height,
                consensus_height,
            ))
            .unwrap()
        }
        /// Setter for `client_id`.
        pub fn with_client_id(self, client_id: ClientId) -> MsgConnectionOpenTry {
            MsgConnectionOpenTry {
                client_id_on_b: client_id,
                ..self
            }
        }
    }

    /// Returns a dummy `RawMsgConnectionOpenTry` with parametrized heights. The parameter
    /// `proof_height` represents the height, on the source chain, at which this chain produced the
    /// proof. Parameter `consensus_height` represents the height of destination chain which a
    /// client on the source chain stores.
    pub fn get_dummy_raw_msg_conn_open_try(
        proof_height: u64,
        consensus_height: u64,
    ) -> RawMsgConnectionOpenTry {
        let client_state_height = Height::new(0, consensus_height).unwrap();

        #[allow(deprecated)]
        RawMsgConnectionOpenTry {
            client_id: ClientId::default().to_string(),
            previous_connection_id: ConnectionId::default().to_string(),
            client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
            counterparty: Some(get_dummy_raw_counterparty(Some(0))),
            delay_period: 0,
            counterparty_versions: get_compatible_versions()
                .iter()
                .map(|v| v.clone().into())
                .collect(),
            proof_init: get_dummy_proof(),
            proof_height: Some(RawHeight {
                revision_number: 0,
                revision_height: proof_height,
            }),
            proof_consensus: get_dummy_proof(),
            consensus_height: Some(RawHeight {
                revision_number: 0,
                revision_height: consensus_height,
            }),
            proof_client: get_dummy_proof(),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::client::v1::Height;
    use ibc_proto::ibc::core::connection::v1::Counterparty as RawCounterparty;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;

    use crate::core::ics03_connection::msgs::conn_open_try::test_util::get_dummy_raw_msg_conn_open_try;
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;

    #[test]
    fn parse_connection_open_try_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenTry,
            want_pass: bool,
        }

        let default_try_msg = get_dummy_raw_msg_conn_open_try(10, 34);

        let tests: Vec<Test> =
            vec![
                Test {
                    name: "Good parameters".to_string(),
                    raw: default_try_msg.clone(),
                    want_pass: true,
                },
                Test {
                    name: "Bad client id, name too short".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        client_id: "client".to_string(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad destination connection id, name too long".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty: Some(RawCounterparty {
                            connection_id:
                            "abcdasdfasdfsdfasfdwefwfsdfsfsfasfwewvxcvdvwgadvaadsefghijklmnopqrstu"
                                .to_string(),
                            ..get_dummy_raw_counterparty(Some(0))
                        }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Correct destination client id with lower/upper case and special chars"
                        .to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty: Some(RawCounterparty {
                            client_id: "ClientId_".to_string(),
                            ..get_dummy_raw_counterparty(Some(0))
                        }),
                        ..default_try_msg.clone()
                    },
                    want_pass: true,
                },
                Test {
                    name: "Bad counterparty versions, empty versions vec".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty_versions: Vec::new(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad counterparty versions, empty version string".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty_versions: Vec::new(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad proof height, height is 0".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_height: Some(Height { revision_number: 1, revision_height: 0 }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad consensus height, height is 0".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_height: Some(Height { revision_number: 1, revision_height: 0 }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Empty proof".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_init: b"".to_vec(),
                        ..default_try_msg
                    },
                    want_pass: false,
                }
            ]
            .into_iter()
            .collect();

        for test in tests {
            let msg = MsgConnectionOpenTry::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenTry::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_conn_open_try(10, 34);
        let msg = MsgConnectionOpenTry::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenTry::from(msg.clone());
        let msg_back = MsgConnectionOpenTry::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
