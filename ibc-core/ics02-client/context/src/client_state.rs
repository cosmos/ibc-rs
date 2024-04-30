//! Defines `ClientState`, the core type to be implemented by light clients

use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::{Height, Status};
use ibc_core_commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core_host_types::identifiers::{ClientId, ClientType};
use ibc_core_host_types::path::Path;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;

use crate::context::{ClientExecutionContext, ClientValidationContext};
use crate::Convertible;

/// `ClientState` methods needed in both validation and execution.
///
/// They do not require access to a client `ValidationContext` nor
/// `ExecutionContext`.
pub trait ClientStateCommon: Convertible<Any> {
    /// Performs basic validation on the `consensus_state`.
    ///
    /// Notably, an implementation should verify that it can properly
    /// deserialize the object into the expected format.
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError>;

    /// Type of client associated with this state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Latest height the client was updated to
    fn latest_height(&self) -> Height;

    /// Validate that the client is at a sufficient height
    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError>;

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
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError>;

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

/// `ClientState` methods which require access to the client's validation
/// context
///
/// The generic type `V` enables light client developers to expand the set of
/// methods available under the [`ClientValidationContext`] trait and use them in
/// their implementation for validating a client state transition.
///
/// ```ignore
/// impl<V> ClientStateValidation<V> for MyClientState
/// where
///     V: ClientValidationContext + MyValidationContext,
/// {
///   // `MyValidationContext` methods available
/// }
///
/// trait MyValidationContext {
///   // My Context methods
/// }
/// ```
pub trait ClientStateValidation<V>: ClientStateCommon
where
    V: ClientValidationContext,
{
    /// verify_client_message must verify a client_message. A client_message
    /// could be a Header, Misbehaviour. It must handle each type of
    /// client_message appropriately. Calls to check_for_misbehaviour,
    /// update_state, and update_state_on_misbehaviour will assume that the
    /// content of the client_message has been verified and can be trusted. An
    /// error should be returned if the client_message fails to verify.
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError>;

    /// Checks for evidence of a misbehaviour in Header or Misbehaviour type. It
    /// assumes the client_message has already been verified.
    fn check_for_misbehaviour(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError>;

    /// Returns the status of the client. Only Active clients are allowed to process packets.
    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError>;

    /// Verifies whether the calling (subject) client state matches the substitute
    /// client state for the purposes of client recovery.
    ///
    /// Note that this validation function does not need to perform *all* of the
    /// validation steps necessary to confirm client recovery. Some checks, such
    /// as checking that the subject client state's latest height < the substitute
    /// client's latest height, as well as checking that the subject client is
    /// inactive and that the substitute client is active, are performed by the
    /// `validate` function in the `recover_client` module at the ics02-client
    /// level.
    ///
    /// Returns `Ok` if the subject and substitute client states match, `Err` otherwise.
    fn check_substitute(&self, ctx: &V, substitute_client_state: Any) -> Result<(), ClientError>;
}

/// `ClientState` methods which require access to the client's
/// `ExecutionContext`.
///
/// The generic type `E` enables light client developers to expand the set of
/// methods available under the [`ClientExecutionContext`] trait and use them in
/// their implementation for executing a client state transition.
pub trait ClientStateExecution<E>: ClientStateValidation<E>
where
    E: ClientExecutionContext,
{
    /// Initialises the client with the initial client and consensus states.
    ///
    /// Most clients will want to call `E::store_client_state` and
    /// `E::store_consensus_state`.
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError>;

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
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError>;

    /// update_state_on_misbehaviour should perform appropriate state changes on
    /// a client state given that misbehaviour has been detected and verified
    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError>;

    /// Update the client state and consensus state in the store with the upgraded ones.
    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError>;

    /// Update the subject client using the `substitute_client_state` in response
    /// to a successful client recovery.
    fn update_on_recovery(
        &self,
        ctx: &mut E,
        subject_client_id: &ClientId,
        substitute_client_state: Any,
        substitute_consensus_state: Any,
    ) -> Result<(), ClientError>;
}

/// Primary client trait. Defines all the methods that clients must implement.
///
/// `ClientState` is broken up in 3 separate traits to avoid needing to use
/// fully qualified syntax for every method call (see ADR 7 for more details).
/// One only needs to implement [`ClientStateCommon`], [`ClientStateValidation`]
/// and [`ClientStateExecution`]; a blanket implementation will automatically
/// implement `ClientState`.
///
/// Refer to [`ClientStateValidation`] and [`ClientStateExecution`] to learn
/// more about what both generic parameters represent.
pub trait ClientState<V: ClientValidationContext, E: ClientExecutionContext>:
    Send + Sync + ClientStateCommon + ClientStateValidation<V> + ClientStateExecution<E>
{
}

impl<V: ClientValidationContext, E: ClientExecutionContext, T> ClientState<V, E> for T where
    T: Send + Sync + ClientStateCommon + ClientStateValidation<V> + ClientStateExecution<E>
{
}
