use core::marker::{Send, Sync};
use core::time::Duration;

use dyn_clone::DynClone;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqRecvPath,
};
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

    /// Freeze status of the client
    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    /// Frozen height of the client
    fn frozen_height(&self) -> Option<Height>;

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

    /// Verification functions as specified in:
    /// <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics>
    ///
    /// Verify a `proof` that the consensus state of a given client (at height `consensus_height`)
    /// matches the input `consensus_state`. The parameter `counterparty_height` represent the
    /// height of the counterparty chain that this proof assumes (i.e., the height at which this
    /// proof was computed).
    fn verify_client_consensus_state(
        &self,
        proof_height: Height,
        counterparty_prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_cons_state_path: &ClientConsensusStatePath,
        expected_consensus_state: &dyn ConsensusState,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that a connection state matches that of the input `connection_end`.
    fn verify_connection_state(
        &self,
        proof_height: Height,
        counterparty_prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        counterparty_conn_path: &ConnectionPath,
        expected_counterparty_connection_end: &ConnectionEnd,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that a channel state matches that of the input `channel_end`.
    fn verify_channel_state(
        &self,
        proof_height: Height,
        counterparty_prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        counterparty_chan_end_path: &ChannelEndPath,
        expected_counterparty_channel_end: &ChannelEnd,
    ) -> Result<(), ClientError>;

    /// Verify the client state for this chain that it is stored on the counterparty chain.
    fn verify_client_full_state(
        &self,
        proof_height: Height,
        counterparty_prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_state_path: &ClientStatePath,
        expected_client_state: Any,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that a packet has been committed.
    #[allow(clippy::too_many_arguments)]
    fn verify_packet_data(
        &self,
        ctx: &dyn ValidationContext,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that a packet has been committed.
    #[allow(clippy::too_many_arguments)]
    fn verify_packet_acknowledgement(
        &self,
        ctx: &dyn ValidationContext,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        ack_path: &AckPath,
        ack: AcknowledgementCommitment,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that of the next_seq_received.
    #[allow(clippy::too_many_arguments)]
    fn verify_next_sequence_recv(
        &self,
        ctx: &dyn ValidationContext,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        seq_recv_path: &SeqRecvPath,
        sequence: Sequence,
    ) -> Result<(), ClientError>;

    /// Verify a `proof` that a packet has not been received.
    fn verify_packet_receipt_absence(
        &self,
        ctx: &dyn ValidationContext,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        receipt_path: &ReceiptPath,
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
