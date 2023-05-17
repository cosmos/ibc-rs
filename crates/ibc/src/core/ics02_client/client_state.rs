//! Defines `ClientState`, the core type to be implemented by light clients

use core::marker::{Send, Sync};
use core::time::Duration;

use dyn_clone::DynClone;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;

use crate::clients::AsAny;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::Path;
use crate::erased::ErasedSerialize;
use crate::prelude::*;
use crate::Height;

use super::consensus_state::ConsensusState;

use crate::core::{ExecutionContext, ValidationContext};

/// The core light client type.
pub trait ClientState:
    AsAny
    + sealed::ErasedPartialEqClientState
    + DynClone
    + ErasedSerialize
    + ErasedProtobuf<Any, Error = ClientError>
    + core::fmt::Debug
    + Send
    + Sync
{
    /// Type of client associated with this state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Latest height the client was updated to
    fn latest_height(&self) -> Height;

    /// Validate that the client is at a sufficient height
    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError>;

    /// Assert that the client is not frozen
    fn confirm_not_frozen(&self) -> Result<(), ClientError>;

    /// Check if the state is expired when `elapsed` time has passed since the latest consensus
    /// state timestamp
    fn expired(&self, elapsed: Duration) -> bool;

    /// Convert into a boxed trait object
    fn into_box(self) -> Box<dyn ClientState>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError>;

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
    ) -> Result<(), ClientError>;

    /// Checks for evidence of a misbehaviour in Header or Misbehaviour type. It
    /// assumes the client_message has already been verified.
    fn check_for_misbehaviour(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError>;

    /// Updates and stores as necessary any associated information for an IBC
    /// client, such as the ClientState and corresponding ConsensusState. Upon
    /// successful update, a list of consensus heights is returned. It assumes
    /// the client_message has already been verified.
    ///
    /// Note that `header` is the field associated with `UpdateKind::UpdateClient`.
    ///
    /// Post-condition: on success, the return value MUST contain at least one
    /// height.
    fn update_state(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError>;

    /// update_state_on_misbehaviour should perform appropriate state changes on
    /// a client state given that misbehaviour has been detected and verified
    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError>;

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
        proof_upgrade_client: MerkleProof,
        proof_upgrade_consensus_state: MerkleProof,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError>;

    // Update the client state and consensus state in the store with the upgraded ones.
    fn update_state_with_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<UpdatedState, ClientError>;

    // Verify_membership is a generic proof verification method which verifies a
    // proof of the existence of a value at a given Path.
    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError>;

    // Verify_non_membership is a generic proof verification method which
    // verifies the absence of a given commitment.
    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError>;
}

// Implements `Clone` for `Box<dyn ClientState>`
dyn_clone::clone_trait_object!(ClientState);

// Implements `serde::Serialize` for all types that have ClientState as supertrait
#[cfg(feature = "serde")]
erased_serde::serialize_trait_object!(ClientState);

impl PartialEq for dyn ClientState {
    fn eq(&self, other: &Self) -> bool {
        self.eq_client_state(other)
    }
}

// see https://github.com/rust-lang/rust/issues/31740
impl PartialEq<&Self> for Box<dyn ClientState> {
    fn eq(&self, other: &&Self) -> bool {
        self.eq_client_state(other.as_ref())
    }
}

/// `UpdateKind` represents the 2 ways that a client can be updated
/// in IBC: either through a `MsgUpdateClient`, or a `MsgSubmitMisbehaviour`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateKind {
    /// this is the typical scenario where a new header is submitted to the client
    /// to update the client. Note that light clients are free to define the type
    /// of the object used to update them (e.g. could be a list of headers).
    UpdateClient,
    /// this is the scenario where misbehaviour is submitted to the client
    /// (e.g 2 headers with the same height in Tendermint)
    SubmitMisbehaviour,
}

/// Represents the updated client and consensus states after a client upgrade
pub struct UpdatedState {
    pub client_state: Box<dyn ClientState>,
    pub consensus_state: Box<dyn ConsensusState>,
}

mod sealed {
    use super::*;

    pub trait ErasedPartialEqClientState {
        fn eq_client_state(&self, other: &dyn ClientState) -> bool;
    }

    impl<CS> ErasedPartialEqClientState for CS
    where
        CS: ClientState + PartialEq,
    {
        fn eq_client_state(&self, other: &dyn ClientState) -> bool {
            other
                .as_any()
                .downcast_ref::<CS>()
                .map_or(false, |h| self == h)
        }
    }
}
