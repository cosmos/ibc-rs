use core::time::Duration;

use ibc_client_tendermint_types::{
    AllowUpdate, ClientState as ClientStateType, Header as TmHeader,
    Misbehaviour as TmMisbehaviour, TENDERMINT_HEADER_TYPE_URL, TENDERMINT_MISBEHAVIOUR_TYPE_URL,
};
use ibc_core_client::context::client_state::ClientStateValidation;
use ibc_core_client::types::error::ClientError;
use ibc_core_client::types::{Height, Status};
use ibc_core_host::types::identifiers::{ChainId, ClientId};
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;

use super::{
    check_for_misbehaviour_misbehavior, check_for_misbehaviour_update_client, ClientState,
};
use crate::client_state::{verify_header, verify_misbehaviour};
use crate::context::{
    ConsensusStateConverter, DefaultVerifier, TmVerifier, ValidationContext as TmValidationContext,
};

impl<V> ClientStateValidation<V> for ClientState
where
    V: TmValidationContext,
    // V::ClientStateRef: TryInto<ClientStateType, Error = ClientError>,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    /// The default verification logic exposed by ibc-rs simply delegates to a
    /// standalone `verify_client_message` function. This is to make it as simple
    /// as possible for those who merely need the `DefaultVerifier` behaviour, as
    /// well as those who require custom verification logic.
    fn verify_client_message(
        &self,
        ctx: &V,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        verify_client_message(
            self.inner(),
            ctx,
            client_id,
            client_message,
            &DefaultVerifier,
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

    fn check_substitute(&self, ctx: &V, substitute_client_state: Any) -> Result<(), ClientError> {
        check_substitute(self.inner(), ctx, substitute_client_state)
    }
}

/// Verify the client message as part of the client state validation process.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateValidation`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible. It mostly adheres to
/// the same signature as the `ClientStateValidation::verify_client_message`
/// function, except for an additional `verifier` parameter that allows users
/// who require custom verification logic to easily pass in their own verifier
/// implementation.
pub fn verify_client_message<V>(
    client_state: &ClientStateType,
    ctx: &V,
    client_id: &ClientId,
    client_message: Any,
    verifier: &impl TmVerifier,
) -> Result<(), ClientError>
where
    V: TmValidationContext,
    // V::ClientStateRef: TryInto<ClientStateType, Error = ClientError>,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    match client_message.type_url.as_str() {
        TENDERMINT_HEADER_TYPE_URL => {
            let header = TmHeader::try_from(client_message)?;
            verify_header(client_state, ctx, client_id, &header, verifier)
        }
        TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = TmMisbehaviour::try_from(client_message)?;
            verify_misbehaviour(client_state, ctx, client_id, &misbehaviour, verifier)
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
    V: TmValidationContext,
    // V::ClientStateRef: TryInto<ClientStateType, Error = ClientError>,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    match client_message.type_url.as_str() {
        TENDERMINT_HEADER_TYPE_URL => {
            let header = TmHeader::try_from(client_message)?;
            check_for_misbehaviour_update_client(client_state, ctx, client_id, header)
        }
        TENDERMINT_MISBEHAVIOUR_TYPE_URL => {
            let misbehaviour = TmMisbehaviour::try_from(client_message)?;
            check_for_misbehaviour_misbehavior(&misbehaviour)
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
    V: TmValidationContext,
    // V::ClientStateRef: TryInto<ClientStateType, Error = ClientError>,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    if client_state.is_frozen() {
        return Ok(Status::Frozen);
    }

    let latest_consensus_state = {
        match ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height.revision_number(),
            client_state.latest_height.revision_height(),
        )) {
            Ok(cs) => cs.try_into()?,
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
    _ctx: &V,
    substitute_client_state: Any,
) -> Result<(), ClientError>
where
    V: TmValidationContext,
    // V::ClientStateRef: TryInto<ClientStateType, Error = ClientError>,
    V::ConsensusStateRef: ConsensusStateConverter,
{
    let subject = ClientStateType {
        latest_height: Height::new(0, 1).expect("Panic while creating a Height { 0, 1 }"),
        frozen_height: None,
        trusting_period: Duration::ZERO,
        chain_id: ChainId::new("").expect("Panic while creating an empty chain ID"),
        allow_update: AllowUpdate {
            after_expiry: true,
            after_misbehaviour: true,
        },
        ..subject_client_state.clone()
    };

    let substitute_client_state = ClientStateType::try_from(substitute_client_state)?;

    let substitute = ClientStateType {
        latest_height: Height::new(0, 1).expect("Panic while creating a Height { 0, 1 }"),
        frozen_height: None,
        trusting_period: Duration::ZERO,
        chain_id: ChainId::new("").expect("Panic while creating an empty chain ID"),
        allow_update: AllowUpdate {
            after_expiry: true,
            after_misbehaviour: true,
        },
        ..substitute_client_state.clone()
    };

    if subject != substitute {
        return Err(ClientError::ClientRecoveryStateMismatch);
    }

    Ok(())
}
