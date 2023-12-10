use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::ConnectionId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;
use ibc_proto::Protobuf;

use crate::error::ConnectionError;
use crate::version::Version;

pub const CONN_OPEN_ACK_TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenAck";

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgConnectionOpenAck {
    /// ConnectionId that chain A has chosen for it's ConnectionEnd
    pub conn_id_on_a: ConnectionId,
    /// ConnectionId that chain B has chosen for it's ConnectionEnd
    pub conn_id_on_b: ConnectionId,
    /// ClientState of client tracking chain A on chain B
    pub client_state_of_a_on_b: Any,
    /// proof of ConnectionEnd stored on Chain B during ConnOpenTry
    pub proof_conn_end_on_b: CommitmentProofBytes,
    /// proof of ClientState tracking chain A on chain B
    pub proof_client_state_of_a_on_b: CommitmentProofBytes,
    /// proof that chain B has stored ConsensusState of chain A on its client
    pub proof_consensus_state_of_a_on_b: CommitmentProofBytes,
    /// Height at which all proofs in this message were taken
    pub proofs_height_on_b: Height,
    /// height of latest header of chain A that updated the client on chain B
    pub consensus_height_of_a_on_b: Height,
    pub version: Version,
    pub signer: Signer,
    /// optional proof of host state machines (chain A) that are unable to
    /// introspect their own consensus state
    pub proof_consensus_state_of_a: Option<CommitmentProofBytes>,
}

impl Protobuf<RawMsgConnectionOpenAck> for MsgConnectionOpenAck {}

impl TryFrom<RawMsgConnectionOpenAck> for MsgConnectionOpenAck {
    type Error = ConnectionError;

    fn try_from(msg: RawMsgConnectionOpenAck) -> Result<Self, Self::Error> {
        Ok(Self {
            conn_id_on_a: msg
                .connection_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            conn_id_on_b: msg
                .counterparty_connection_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            client_state_of_a_on_b: msg
                .client_state
                .ok_or(ConnectionError::MissingClientState)?,
            version: msg
                .version
                .ok_or(ConnectionError::EmptyVersions)?
                .try_into()?,
            proof_conn_end_on_b: msg
                .proof_try
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_client_state_of_a_on_b: msg
                .proof_client
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_consensus_state_of_a_on_b: msg
                .proof_consensus
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proofs_height_on_b: msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingProofHeight)?,
            consensus_height_of_a_on_b: msg
                .consensus_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingConsensusHeight)?,
            signer: msg.signer.into(),
            proof_consensus_state_of_a: if msg.host_consensus_state_proof.is_empty() {
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

impl From<MsgConnectionOpenAck> for RawMsgConnectionOpenAck {
    fn from(msg: MsgConnectionOpenAck) -> Self {
        RawMsgConnectionOpenAck {
            connection_id: msg.conn_id_on_a.as_str().to_string(),
            counterparty_connection_id: msg.conn_id_on_b.as_str().to_string(),
            client_state: Some(msg.client_state_of_a_on_b),
            proof_height: Some(msg.proofs_height_on_b.into()),
            proof_try: msg.proof_conn_end_on_b.into(),
            proof_client: msg.proof_client_state_of_a_on_b.into(),
            proof_consensus: msg.proof_consensus_state_of_a_on_b.into(),
            consensus_height: Some(msg.consensus_height_of_a_on_b.into()),
            version: Some(msg.version.into()),
            signer: msg.signer.to_string(),
            host_consensus_state_proof: match msg.proof_consensus_state_of_a {
                Some(proof) => proof.into(),
                None => vec![],
            },
        }
    }
}
