use ibc_client_tendermint_types::{
    ClientState as ClientStateType, ConsensusState as ConsensusStateType, Header as TmHeader,
    Misbehaviour as TmMisbehaviour, TENDERMINT_HEADER_TYPE_URL, TENDERMINT_MISBEHAVIOUR_TYPE_URL,
};
use ibc_core_client::context::client_state::ClientStateValidation;
use ibc_core_client::context::{Convertible, ExtClientValidationContext};
use ibc_core_client::types::error::ClientError;
use ibc_core_client::types::Status;
use ibc_core_host::types::identifiers::ClientId;
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;
use tendermint::crypto::default::Sha256;
use tendermint::crypto::Sha256 as Sha256Trait;
use tendermint::merkle::MerkleHash;
use tendermint_light_client_verifier::{ProdVerifier, Verifier};

use super::{check_for_misbehaviour_on_misbehavior, check_for_misbehaviour_on_update, ClientState};
use crate::client_state::{verify_header, verify_misbehaviour};

impl<V> ClientStateValidation<V> for ClientState
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<V::ConsensusStateRef>>::Error: Into<ClientError>,
{
    /// The default verification logic exposed by ibc-rs simply delegates to a
    /// standalone `verify_client_message` function. This is to make it as
    /// simple as possible for those who merely need the default
    /// [`ProdVerifier`] behaviour, as well as those who require custom
    /// verification logic.
    ///
    /// In a situation where the Tendermint [`ProdVerifier`] doesn't provide the
    /// desired outcome, users should define a custom verifier struct and then
    /// implement the [`Verifier`] trait for it.
    ///
    /// In order to wire up the custom verifier, create a newtype `ClientState`
    /// wrapper similar to [`ClientState`] and implement all client state traits
    /// for it. For method implementation, the simplest way is to import and
    /// call their analogous standalone versions under the
    /// [`crate::client_state`] module, unless bespoke logic is desired for any
    /// of those functions. Then, when it comes to implementing the
    /// `verify_client_message` method, use the [`verify_client_message`]
    /// function and pass your custom verifier object as the `verifier`
    /// parameter.
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        verify_client_message::<V, Sha256>(
            self.inner(),
            ctx,
            client_id,
            client_message,
            &ProdVerifier::default(),
        )
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError> {
        check_for_misbehaviour(self.inner(), ctx, client_id, client_message)
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        status(self.inner(), ctx, client_id)
    }

    fn check_substitute(&self, _ctx: &V, substitute_client_state: Any) -> Result<(), ClientError> {
        check_substitute::<V>(self.inner(), substitute_client_state)
    }
}

/// Verify the client message as part of the client state validation process.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateValidation`] trait, but has been made a standalone function in
/// order to make the ClientState APIs more flexible. It mostly adheres to the
/// same signature as the `ClientStateValidation::verify_client_message`
/// function, except for an additional `verifier` parameter that allows users
/// who require custom verification logic to easily pass in their own verifier
/// implementation.
pub fn verify_client_message<V, H>(
    client_state: &ClientStateType,
    ctx: &V,
    client_id: &ClientId,
    client_message: Any,
    verifier: &impl Verifier,
) -> Result<(), ClientError>
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<V::ConsensusStateRef>>::Error: Into<ClientError>,
    H: MerkleHash + Sha256Trait + Default,
{
    match client_message.type_url.as_str() {
        TENDERMINT_HEADER_TYPE_URL => {
            let header = TmHeader::try_from(client_message)?;
            verify_header::<V, H>(
                ctx,
                &header,
                client_id,
                client_state.chain_id(),
                &client_state.as_light_client_options()?,
                verifier,
            )
        }
        TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = TmMisbehaviour::try_from(client_message)?;
            verify_misbehaviour::<V, H>(
                ctx,
                &misbehaviour,
                client_id,
                client_state.chain_id(),
                &client_state.as_light_client_options()?,
                verifier,
            )
        }
        _ => Err(ClientError::InvalidUpdateClientMessage),
    }
}

/// Check for misbehaviour on the client state as part of the client state
/// validation process.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateValidation`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
///
/// This method covers the following cases:
///
/// 1 - fork:
/// Assumes at least one consensus state before the fork point exists. Let
/// existing consensus states on chain B be: [Sn,.., Sf, Sf-1, S0] with
/// `Sf-1` being the most recent state before fork. Chain A is queried for a
/// header `Hf'` at `Sf.height` and if it is different than the `Hf` in the
/// event for the client update (the one that has generated `Sf` on chain),
/// then the two headers are included in the evidence and submitted. Note
/// that in this case the headers are different but have the same height.
///
/// 2 - BFT time violation for unavailable header (a.k.a. Future Lunatic
/// Attack or FLA):
/// Some header with a height that is higher than the latest height on A has
/// been accepted and a consensus state was created on B. Note that this
/// implies that the timestamp of this header must be within the
/// `clock_drift` of the client. Assume the client on B has been updated
/// with `h2`(not present on/ produced by chain A) and it has a timestamp of
/// `t2` that is at most `clock_drift` in the future. Then the latest header
/// from A is fetched, let it be `h1`, with a timestamp of `t1`. If `t1 >=
/// t2` then evidence of misbehavior is submitted to A.
///
/// 3 - BFT time violation for existing headers:
/// Ensure that consensus state times are monotonically increasing with
/// height.
pub fn check_for_misbehaviour<V>(
    client_state: &ClientStateType,
    ctx: &V,
    client_id: &ClientId,
    client_message: Any,
) -> Result<bool, ClientError>
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<V::ConsensusStateRef>>::Error: Into<ClientError>,
{
    match client_message.type_url.as_str() {
        TENDERMINT_HEADER_TYPE_URL => {
            let header = TmHeader::try_from(client_message)?;
            check_for_misbehaviour_on_update(ctx, header, client_id, &client_state.latest_height)
        }
        TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = TmMisbehaviour::try_from(client_message)?;
            check_for_misbehaviour_on_misbehavior(misbehaviour.header1(), misbehaviour.header2())
        }
        _ => Err(ClientError::InvalidUpdateClientMessage),
    }
}

/// Query the status of the client state.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateValidation`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn status<V>(
    client_state: &ClientStateType,
    ctx: &V,
    client_id: &ClientId,
) -> Result<Status, ClientError>
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<V::ConsensusStateRef>>::Error: Into<ClientError>,
{
    if client_state.is_frozen() {
        return Ok(Status::Frozen);
    }

    let latest_consensus_state: ConsensusStateType = {
        match ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height.revision_number(),
            client_state.latest_height.revision_height(),
        )) {
            Ok(cs) => cs.try_into().map_err(Into::into)?,
            // if the client state does not have an associated consensus state for its latest height
            // then it must be expired
            Err(_) => return Ok(Status::Expired),
        }
    };

    // Note: if the `duration_since()` is `None`, indicating that the latest
    // consensus state is in the future, then we don't consider the client
    // to be expired.
    let now = ctx.host_timestamp()?;

    if let Some(elapsed_since_latest_consensus_state) =
        now.duration_since(&latest_consensus_state.timestamp().into())
    {
        if elapsed_since_latest_consensus_state > client_state.trusting_period {
            return Ok(Status::Expired);
        }
    }

    Ok(Status::Active)
}

/// Check that the subject and substitute client states match as part of
/// the client recovery validation step.
///
/// The subject and substitute client states match if all their respective
/// client state parameters match except for frozen height, latest height,
/// trusting period, and chain ID.
pub fn check_substitute<V>(
    subject_client_state: &ClientStateType,
    substitute_client_state: Any,
) -> Result<(), ClientError>
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
{
    let ClientStateType {
        latest_height: _,
        frozen_height: _,
        trusting_period: _,
        chain_id: _,
        allow_update: _,
        trust_level: subject_trust_level,
        unbonding_period: subject_unbonding_period,
        max_clock_drift: subject_max_clock_drift,
        proof_specs: subject_proof_specs,
        upgrade_path: subject_upgrade_path,
    } = subject_client_state;

    let substitute_client_state = ClientStateType::try_from(substitute_client_state)?;

    let ClientStateType {
        latest_height: _,
        frozen_height: _,
        trusting_period: _,
        chain_id: _,
        allow_update: _,
        trust_level: substitute_trust_level,
        unbonding_period: substitute_unbonding_period,
        max_clock_drift: substitute_max_clock_drift,
        proof_specs: substitute_proof_specs,
        upgrade_path: substitute_upgrade_path,
    } = substitute_client_state;

    (subject_trust_level == &substitute_trust_level
        && subject_unbonding_period == &substitute_unbonding_period
        && subject_max_clock_drift == &substitute_max_clock_drift
        && subject_proof_specs == &substitute_proof_specs
        && subject_upgrade_path == &substitute_upgrade_path)
        .then_some(())
        .ok_or(ClientError::ClientRecoveryStateMismatch)
}
