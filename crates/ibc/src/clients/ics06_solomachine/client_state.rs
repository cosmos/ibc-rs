use crate::clients::ics06_solomachine::consensus_state::ConsensusState as SoloMachineConsensusState;
use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::client_state::{ClientState as Ics2ClientState, UpdatedState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::msgs::update_client::UpdateKind;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::Path;
use crate::core::{ExecutionContext, ValidationContext};
use crate::prelude::*;
use crate::Height;
use core::time::Duration;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_proto::ibc::lightclients::solomachine::v1::ClientState as RawSolClientState;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub const SOLOMACHINE_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.solomachine.v1.ClientState";

/// ClientState defines a solo machine client that tracks the current consensus
/// state and if the client is frozen.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct ClientState {
    /// latest sequence of the client state
    pub sequence: Height,
    /// frozen sequence of the solo machine
    pub frozen_sequence: Option<Height>,
    pub consensus_state: Option<SoloMachineConsensusState>,
    /// when set to true, will allow governance to update a solo machine client.
    /// The client will be unfrozen if it is frozen.
    pub allow_update_after_proposal: bool,
}

impl ClientState {
    /// Create a new ClientState Instance.
    pub fn new(
        sequence: Height,
        frozen_sequence: Option<Height>,
        consensus_state: Option<SoloMachineConsensusState>,
        allow_update_after_proposal: bool,
    ) -> Self {
        Self {
            sequence,
            frozen_sequence,
            consensus_state,
            allow_update_after_proposal,
        }
    }

    /// Return exported.Height to satisfy ClientState interface
    /// Revision number is always 0 for a solo-machine.
    pub fn latest_height(&self) -> Height {
        self.sequence
    }
}

impl Ics2ClientState for ClientState {
    /// Return the chain identifier which this client is serving (i.e., the client is verifying
    /// consensus states from this chain).
    fn chain_id(&self) -> ChainId {
        todo!()
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
        todo!()
    }

    /// Assert that the client is not frozen
    fn confirm_not_frozen(&self) -> Result<(), ClientError> {
        if let Some(frozen_height) = self.frozen_sequence {
            return Err(ClientError::ClientFrozen {
                description: format!("the client is frozen at height {frozen_height}"),
            });
        }
        Ok(())
    }

    /// Check if the state is expired when `elapsed` time has passed since the latest consensus
    /// state timestamp
    fn expired(&self, elapsed: Duration) -> bool {
        todo!()
    }

    /// Helper function to verify the upgrade client procedure.
    /// Resets all fields except the blockchain-specific ones,
    /// and updates the given fields.
    // ref: https://github.com/cosmos/ibc-go/blob/fa9418f5ba39de38d995ec83d9abd289dfab5f9f/modules/light-clients/06-solomachine/client_state.go#L75
    fn zero_custom_fields(&mut self) {
        // zero_custom_fields is not implemented for solo machine
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError> {
        todo!()
    }

    /// verify_client_message must verify a client_message. A client_message
    /// could be a Header, Misbehaviour. It must handle each type of
    /// client_message appropriately. Calls to check_for_misbehaviour,
    /// update_state, and update_state_on_misbehaviour will assume that the
    /// content of the client_message has been verified and can be trusted. An
    /// error should be returned if the client_message fails to verify.
    fn verify_client_message(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        todo!()
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
        todo!()
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
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<Vec<Height>, ClientError> {
        todo!()
    }

    /// update_state_on_misbehaviour should perform appropriate state changes on
    /// a client state given that misbehaviour has been detected and verified
    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        todo!()
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
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: RawMerkleProof,
        proof_upgrade_consensus_state: RawMerkleProof,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        todo!()
    }

    // Update the client state and consensus state in the store with the upgraded ones.
    fn update_state_with_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<UpdatedState, ClientError> {
        todo!()
    }

    // Verify_membership is a generic proof verification method which verifies a
    // proof of the existence of a value at a given Path.
    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        todo!()
    }

    // Verify_non_membership is a generic proof verification method which
    // verifies the absence of a given commitment.
    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        todo!()
    }
}

impl Protobuf<RawSolClientState> for ClientState {}

impl TryFrom<RawSolClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawSolClientState) -> Result<Self, Self::Error> {
        let sequence = Height::new(0, raw.sequence).map_err(Error::InvalidHeight)?;
        let frozen_sequence = if raw.frozen_sequence == 0 {
            None
        } else {
            Some(Height::new(0, raw.frozen_sequence).map_err(Error::InvalidHeight)?)
        };

        let consensus_state: Option<SoloMachineConsensusState> =
            raw.consensus_state.map(TryInto::try_into).transpose()?;

        Ok(Self {
            sequence,
            frozen_sequence,
            consensus_state,
            allow_update_after_proposal: raw.allow_update_after_proposal,
        })
    }
}

impl From<ClientState> for RawSolClientState {
    fn from(value: ClientState) -> Self {
        let sequence = value.sequence.revision_height();
        let frozen_sequence = if let Some(seq) = value.frozen_sequence {
            seq.revision_height()
        } else {
            0
        };
        let consensus_state = value.consensus_state.map(Into::into);
        Self {
            sequence,
            frozen_sequence,
            consensus_state,
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
            RawSolClientState::decode(buf)
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
            value: Protobuf::<RawSolClientState>::encode_vec(&client_state)
                .expect("encoding to `Any` from `RawSolClientState`"),
        }
    }
}
