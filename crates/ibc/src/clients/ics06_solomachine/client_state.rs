use super::misbehaviour::signature_and_data::SignatureAndData;
use crate::clients::ics06_solomachine::consensus_state::ConsensusState as SmConsensusState;
use crate::clients::ics06_solomachine::error::Error;
use crate::clients::ics06_solomachine::header::Header as SmHeader;
use crate::clients::ics06_solomachine::misbehaviour::Misbehaviour as SmMisbehaviour;
use crate::clients::ics06_solomachine::proof::verify_signature;
use crate::clients::ics06_solomachine::types::sign_bytes::SignBytes;
use crate::clients::ics06_solomachine::types::timestamped_signature_data::TimestampedSignatureData;
use crate::clients::ics06_solomachine::types::DataType;
use crate::core::ics02_client::client_state::UpdateKind;
use crate::core::ics02_client::client_state::{ClientState as Ics2ClientState, UpdatedState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::path::ClientStatePath;
use crate::core::ics24_host::path::Path;
use crate::core::timestamp::Timestamp;
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;
use crate::Height;
use core::time::Duration;
use cosmrs::crypto::PublicKey;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_proto::ibc::lightclients::solomachine::v2::ClientState as RawSmClientState;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub mod misbehaviour;
pub mod update_client;

pub const SOLOMACHINE_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.solomachine.v1.ClientState";

/// ClientState defines a solo machine client that tracks the current consensus
/// state and if the client is frozen.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct ClientState {
    /// latest sequence of the client state
    pub sequence: Height,
    /// frozen sequence of the solo machine
    pub is_frozen: bool,
    pub consensus_state: SmConsensusState,
    /// when set to true, will allow governance to update a solo machine client.
    /// The client will be unfrozen if it is frozen.
    pub allow_update_after_proposal: bool,
}

impl ClientState {
    /// Create a new ClientState Instance.
    pub fn new(
        sequence: Height,
        is_frozen: bool,
        consensus_state: SmConsensusState,
        allow_update_after_proposal: bool,
    ) -> Self {
        Self {
            sequence,
            is_frozen,
            consensus_state,
            allow_update_after_proposal,
        }
    }

    pub fn with_frozen(self) -> Self {
        Self {
            is_frozen: true,
            ..self
        }
    }

    /// Return exported.Height to satisfy ClientState interface
    /// Revision number is always 0 for a solo-machine.
    pub fn latest_height(&self) -> Height {
        self.sequence
    }

    // GetTimestampAtHeight returns the timestamp in nanoseconds of the consensus state at the given height.
    pub fn time_stamp(&self) -> Timestamp {
        self.consensus_state.timestamp
    }

    // Validate performs basic validation of the client state fields.
    pub fn valida_basic(&self) -> Result<(), Error> {
        if self.sequence.revision_height() == 0 {
            return Err(Error::SequenceCannotZero);
        }
        self.consensus_state.valida_basic()
    }

    // produceVerificationArgs perfoms the basic checks on the arguments that are
    // shared between the verification functions and returns the public key of the
    // consensus state, the unmarshalled proof representing the signature and timestamp.
    pub fn produce_verification_args(
        &self,
        proof: &CommitmentProofBytes,
    ) -> Result<(PublicKey, SignatureAndData, Timestamp, Height), Error> {
        let proof = Vec::<u8>::from(proof.clone());
        if proof.is_empty() {
            return Err(Error::ProofCannotEmpty);
        }
        let timestamped_sig_data = TimestampedSignatureData::decode_vec(&proof).map_err(|e| {
            Error::Other(format!(
                "failed to decode proof into type TimestampedSignatureData: {}",
                e
            ))
        })?;

        let timestamp = timestamped_sig_data.timestamp;

        if timestamped_sig_data.signature_data.is_empty() {
            return Err(Error::Other("signature data cannot be empty".into()));
        }

        let signature_and_data = SignatureAndData::decode_vec(&timestamped_sig_data.signature_data)
            .map_err(|_| Error::Other("failed to decode SignatureData".into()))?;

        if self.consensus_state.timestamp > timestamp {
            return Err(Error::Other(format!(
                "the consensus state timestamp is greater than the signature timestamp ({} >= {})",
                self.consensus_state.timestamp, timestamp
            )));
        }

        let sequence = self.latest_height();
        let public_key = self.consensus_state.public_key();
        Ok((public_key, signature_and_data, timestamp, sequence))
    }
}

impl Ics2ClientState for ClientState {
    /// Return the chain identifier which this client is serving (i.e., the client is verifying
    /// consensus states from this chain).
    fn chain_id(&self) -> ChainId {
        // todo(davirian)
        ChainId::default()
    }

    /// ClientType is Solo Machine.
    fn client_type(&self) -> ClientType {
        super::client_type()
    }

    /// latest_height returns the latest sequence number.
    fn latest_height(&self) -> Height {
        self.latest_height()
    }

    /// Check if the given proof has a valid height for the client
    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    /// Assert that the client is not frozen
    fn confirm_not_frozen(&self) -> Result<(), ClientError> {
        if self.is_frozen {
            return Err(ClientError::ClientFrozen {
                description: "the client is frozen".into(),
            });
        }
        Ok(())
    }

    /// Check if the state is expired when `elapsed` time has passed since the latest consensus
    /// state timestamp
    fn expired(&self, _elapsed: Duration) -> bool {
        // todo(davirian)
        false
    }

    /// Helper function to verify the upgrade client procedure.
    /// Resets all fields except the blockchain-specific ones,
    /// and updates the given fields.
    fn zero_custom_fields(&mut self) {
        // ref: https://github.com/cosmos/ibc-go/blob/f32b1052e1357949e6a67685d355c7bcc6242b84/modules/light-clients/06-solomachine/client_state.go#L76
        panic!("ZeroCustomFields is not implemented as the solo machine implementation does not support upgrades.")
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError> {
        SmConsensusState::try_from(consensus_state).map(SmConsensusState::into_box)
    }

    /// verify_client_message must verify a client_message. A client_message
    /// could be a Header, Misbehaviour. It must handle each type of
    /// client_message appropriately. Calls to check_for_misbehaviour,
    /// update_state, and update_state_on_misbehaviour will assume that the
    /// content of the client_message has been verified and can be trusted. An
    /// error should be returned if the client_message fails to verify.
    ///
    /// VerifyClientMessage introspects the provided ClientMessage and checks its validity
    /// A Solomachine Header is considered valid if the currently registered public key has signed over
    /// the new public key with the correct sequence.
    /// A Solomachine Misbehaviour is considered valid if duplicate signatures of the current public key
    /// are found on two different messages at a given sequence.
    fn verify_client_message(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => {
                let header = SmHeader::try_from(client_message)?;
                self.verify_header(ctx, client_id, header)
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SmMisbehaviour::try_from(client_message)?;
                self.verify_misbehaviour(ctx, client_id, misbehaviour)
            }
        }
    }

    /// Checks for evidence of a misbehaviour in Header or Misbehaviour type. It
    /// assumes the client_message has already been verified.
    fn check_for_misbehaviour(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => {
                let header = SmHeader::try_from(client_message)?;
                self.check_for_misbehaviour_update_client(ctx, client_id, header)
            }
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = SmMisbehaviour::try_from(client_message)?;
                self.check_for_misbehaviour_misbehavior(&misbehaviour)
            }
        }
    }

    /// Updates and stores as necessary any associated information for an IBC
    /// client, such as the ClientState and corresponding ConsensusState. Upon
    /// successful update, a list of consensus heights is returned. It assumes
    /// the client_message has already been verified.
    ///
    /// Post-condition: on success, the return value MUST contain at least one
    /// height.
    fn update_state(
        &self,
        _ctx: &mut dyn ExecutionContext,
        _client_id: &ClientId,
        _header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        todo!()
    }

    /// update_state_on_misbehaviour should perform appropriate state changes on
    /// a client state given that misbehaviour has been detected and verified
    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self.clone().with_frozen().into_box();

        ctx.store_client_state(ClientStatePath::new(client_id), frozen_client_state)?;

        Ok(())
    }

    /// Verify the upgraded client and consensus states and validate proofs
    /// against the given root.
    ///
    /// NOTE: proof heights are not included as upgrade to a new revision is
    /// expected to pass only on the last height committed by the current
    /// revision. Clients are responsible for ensuring that the planned last
    /// height of the current revision is somehow encoded in the proof
    /// verification process. This is to ensure that no premature upgrades
    /// occur, since upgrade plans committed to by the counterparty may be
    /// cancelled or modified before the last planned height.
    fn verify_upgrade_client(
        &self,
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
        _proof_upgrade_client: RawMerkleProof,
        _proof_upgrade_consensus_state: RawMerkleProof,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    // Update the client state and consensus state in the store with the upgraded ones.
    fn update_state_with_upgrade_client(
        &self,
        _upgraded_client_state: Any,
        _upgraded_consensus_state: Any,
    ) -> Result<UpdatedState, ClientError> {
        // ref: https://github.com/cosmos/ibc-go/blob/f32b1052e1357949e6a67685d355c7bcc6242b84/modules/light-clients/06-solomachine/client_state.go#L99
        Err(ClientError::Other {
            description: "cannot upgrade solomachine client".into(),
        })
    }

    // Verify_membership is a generic proof verification method which verifies a
    // proof of the existence of a value at a given Path.
    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let (public_key, sig_data, timestamp, sequence) = self.produce_verification_args(proof)?;
        let data_type = match path {
            Path::ClientState(_) => DataType::ClientState,
            Path::ClientConsensusState(_) => DataType::ConsensusState,
            Path::ClientConnection(_) => DataType::ConnectionState,
            Path::Connection(_) => DataType::ConnectionState,
            Path::Ports(_) => DataType::Header,
            Path::ChannelEnd(_) => DataType::ChannelState,
            Path::SeqSend(_) => DataType::Header,
            Path::SeqRecv(_) => DataType::NextSequenceRecv,
            Path::SeqAck(_) => DataType::Header,
            Path::Commitment(_) => DataType::PacketCommitment,
            Path::Ack(_) => DataType::PacketAcknowledgement,
            Path::Receipt(_) => DataType::Header,
            Path::UpgradeClient(_) => DataType::Header,
        };
        let sign_bytes = SignBytes {
            sequence: sequence.revision_height(),
            timestamp,
            diversifier: self.consensus_state.diversifier.clone(),
            data_type,
            data: value,
        };
        let sign_bz = sign_bytes.encode_vec();

        verify_signature(public_key, sign_bz, sig_data).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })
    }

    // Verify_non_membership is a generic proof verification method which
    // verifies the absence of a given commitment.
    //
    // VerifyNonMembership is a generic proof verification method which verifies the absence
    // of a given CommitmentPath at the latest sequence.
    // The caller is expected to construct the full CommitmentPath from a CommitmentPrefix
    // and a standardized path (as defined in ICS 24).
    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        let (public_key, sig_data, timestamp, sequence) = self.produce_verification_args(proof)?;
        let data_type = match path {
            Path::ClientState(_) => DataType::ClientState,
            Path::ClientConsensusState(_) => DataType::ConsensusState,
            Path::ClientConnection(_) => DataType::ConnectionState,
            Path::Connection(_) => DataType::ConnectionState,
            Path::Ports(_) => DataType::Header,
            Path::ChannelEnd(_) => DataType::ChannelState,
            Path::SeqSend(_) => DataType::Header,
            Path::SeqRecv(_) => DataType::NextSequenceRecv,
            Path::SeqAck(_) => DataType::Header,
            Path::Commitment(_) => DataType::PacketCommitment,
            Path::Ack(_) => DataType::PacketAcknowledgement,
            Path::Receipt(_) => DataType::Header,
            Path::UpgradeClient(_) => DataType::Header,
        };
        let sign_bytes = SignBytes {
            sequence: sequence.revision_height(),
            timestamp,
            diversifier: self.consensus_state.diversifier.clone(),
            data_type,
            data: vec![],
        };
        let sign_bz = sign_bytes.encode_vec();

        verify_signature(public_key, sign_bz, sig_data).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })
    }
}

impl Protobuf<RawSmClientState> for ClientState {}

impl TryFrom<RawSmClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawSmClientState) -> Result<Self, Self::Error> {
        let sequence = Height::new(0, raw.sequence).map_err(Error::InvalidHeight)?;

        let consensus_state: SmConsensusState = raw
            .consensus_state
            .ok_or(Error::ConsensusStateIsEmpty)?
            .try_into()?;

        Ok(Self {
            sequence,
            is_frozen: raw.is_frozen,
            consensus_state,
            allow_update_after_proposal: raw.allow_update_after_proposal,
        })
    }
}

impl From<ClientState> for RawSmClientState {
    fn from(value: ClientState) -> Self {
        let sequence = value.sequence.revision_height();

        Self {
            sequence,
            is_frozen: value.is_frozen,
            consensus_state: Some(value.consensus_state.into()),
            allow_update_after_proposal: value.allow_update_after_proposal,
        }
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use bytes::Buf;
        use core::ops::Deref;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, Error> {
            RawSmClientState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            SOLOMACHINE_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOLOMACHINE_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSmClientState>::encode_vec(&client_state),
        }
    }
}
