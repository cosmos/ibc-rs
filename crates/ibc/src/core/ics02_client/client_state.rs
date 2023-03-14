use core::marker::{Send, Sync};
use core::time::Duration;

use dyn_clone::DynClone;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::Path;
use crate::dynamic_typing::AsAny;
use crate::erased::ErasedSerialize;
use crate::prelude::*;
use crate::Height;

use super::consensus_state::ConsensusState;

use crate::core::{ContextError, ValidationContext};

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
    /// Return the chain identifier which this client is serving (i.e., the client is verifying
    /// consensus states from this chain).
    fn chain_id(&self) -> ChainId;

    /// Type of client associated with this state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Latest height the client was updated to
    fn latest_height(&self) -> Height;

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

    /// Frozen height of the client
    fn frozen_height(&self) -> Option<Height>;

    /// Assert that the client is not frozen
    fn assert_not_frozen(&self) -> Result<(), ClientError> {
        if let Some(frozen_height) = self.frozen_height() {
            return Err(ClientError::ClientFrozen { frozen_height });
        }
        Ok(())
    }

    /// Check if the state is expired when `elapsed` time has passed since the latest consensus
    /// state timestamp
    fn expired(&self, elapsed: Duration) -> bool;

    /// Helper function to verify the upgrade client procedure.
    /// Resets all fields except the blockchain-specific ones,
    /// and updates the given fields.
    fn zero_custom_fields(&mut self);

    /// Convert into a boxed trait object
    fn into_box(self) -> Box<dyn ClientState>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError>;

    fn check_header_and_update_state(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        header: Any,
    ) -> Result<UpdatedState, ClientError>;

    fn check_misbehaviour_and_update_state(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        misbehaviour: Any,
    ) -> Result<Box<dyn ClientState>, ContextError>;

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

pub fn downcast_client_state<CS: ClientState>(h: &dyn ClientState) -> Option<&CS> {
    h.as_any().downcast_ref::<CS>()
}

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
