use core::time::Duration;

use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;
use ibc_proto::Protobuf;

use crate::connection::Counterparty;
use crate::error::ConnectionError;
use crate::version::Version;

pub const CONN_OPEN_TRY_TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenTry";

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// optional proof of host state machines (chain B) that are unable to
    /// introspect their own consensus state
    pub proof_consensus_state_of_b: Option<CommitmentProofBytes>,

    #[deprecated(since = "0.22.0")]
    /// Only kept here for proper conversion to/from the raw type
    pub previous_connection_id: String,
}

#[allow(deprecated)]
#[cfg(feature = "borsh")]
mod borsh_impls {
    use borsh::maybestd::io::{self, Read};
    use borsh::{BorshDeserialize, BorshSerialize};

    use super::*;

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct InnerMsgConnectionOpenTry {
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
        pub delay_period_nanos: u64,
        pub signer: Signer,
        /// optional proof of host state machines (chain B) that are unable to
        /// introspect their own consensus state
        pub proof_consensus_state_of_b: Option<CommitmentProofBytes>,

        #[deprecated(since = "0.22.0")]
        /// Only kept here for proper conversion to/from the raw type
        previous_connection_id: String,
    }

    impl BorshSerialize for MsgConnectionOpenTry {
        fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
            let delay_period_nanos: u64 =
                self.delay_period.as_nanos().try_into().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Duration too long: {} nanos", self.delay_period.as_nanos()),
                    )
                })?;

            let inner = InnerMsgConnectionOpenTry {
                client_id_on_b: self.client_id_on_b.clone(),
                client_state_of_b_on_a: self.client_state_of_b_on_a.clone(),
                counterparty: self.counterparty.clone(),
                versions_on_a: self.versions_on_a.clone(),
                proof_conn_end_on_a: self.proof_conn_end_on_a.clone(),
                proof_client_state_of_b_on_a: self.proof_client_state_of_b_on_a.clone(),
                proof_consensus_state_of_b_on_a: self.proof_consensus_state_of_b_on_a.clone(),
                proofs_height_on_a: self.proofs_height_on_a,
                consensus_height_of_b_on_a: self.consensus_height_of_b_on_a,
                delay_period_nanos,
                signer: self.signer.clone(),
                proof_consensus_state_of_b: self.proof_consensus_state_of_b.clone(),
                previous_connection_id: self.previous_connection_id.clone(),
            };

            inner.serialize(writer)
        }
    }

    impl BorshDeserialize for MsgConnectionOpenTry {
        fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
            let inner = InnerMsgConnectionOpenTry::deserialize_reader(reader)?;

            Ok(MsgConnectionOpenTry {
                client_id_on_b: inner.client_id_on_b,
                client_state_of_b_on_a: inner.client_state_of_b_on_a,
                counterparty: inner.counterparty,
                versions_on_a: inner.versions_on_a,
                proof_conn_end_on_a: inner.proof_conn_end_on_a,
                proof_client_state_of_b_on_a: inner.proof_client_state_of_b_on_a,
                proof_consensus_state_of_b_on_a: inner.proof_consensus_state_of_b_on_a,
                proofs_height_on_a: inner.proofs_height_on_a,
                consensus_height_of_b_on_a: inner.consensus_height_of_b_on_a,
                delay_period: Duration::from_nanos(inner.delay_period_nanos),
                signer: inner.signer,
                proof_consensus_state_of_b: inner.proof_consensus_state_of_b,
                previous_connection_id: inner.previous_connection_id,
            })
        }
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
            signer: msg.signer.into(),
            proof_consensus_state_of_b: if msg.host_consensus_state_proof.is_empty() {
                None
            } else {
                Some(
                    msg.host_consensus_state_proof
                        .try_into()
                        .map_err(|_| ConnectionError::InvalidProof)?,
                )
            },
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
            host_consensus_state_proof: match msg.proof_consensus_state_of_b {
                Some(proof) => proof.into(),
                None => vec![],
            },
        }
    }
}
