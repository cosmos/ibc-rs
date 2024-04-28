use ibc_client_tendermint_types::{
    ClientState as ClientStateType, ConsensusState as ConsensusStateType, Header as TmHeader,
};
use ibc_core_client::context::prelude::*;
use ibc_core_client::types::error::ClientError;
use ibc_core_client::types::Height;
use ibc_core_host::types::identifiers::ClientId;
use ibc_core_host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;

use super::ClientState;

impl<E> ClientStateExecution<E> for ClientState
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<E::ConsensusStateRef>>::Error: Into<ClientError>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        initialise(self.inner(), ctx, client_id, consensus_state)
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        update_state(self.inner(), ctx, client_id, header)
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        update_on_misbehaviour(self.inner(), ctx, client_id, client_message)
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        update_on_upgrade(
            self.inner(),
            ctx,
            client_id,
            upgraded_client_state,
            upgraded_consensus_state,
        )
    }

    fn update_on_recovery(
        &self,
        ctx: &mut E,
        subject_client_id: &ClientId,
        substitute_client_state: Any,
        substitute_consensus_state: Any,
    ) -> Result<(), ClientError> {
        let subject_client_state = self.inner().clone();

        update_on_recovery(
            subject_client_state,
            ctx,
            subject_client_id,
            substitute_client_state,
            substitute_consensus_state,
        )
    }
}

/// Seed the host store with initial client and consensus states.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateExecution`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn initialise<E>(
    client_state: &ClientStateType,
    ctx: &mut E,
    client_id: &ClientId,
    consensus_state: Any,
) -> Result<(), ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
{
    let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
    let host_height = ExtClientValidationContext::host_height(ctx)?;

    let tm_consensus_state: ConsensusStateType = consensus_state.try_into()?;

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        client_state.clone().into(),
    )?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            client_id.clone(),
            client_state.latest_height.revision_number(),
            client_state.latest_height.revision_height(),
        ),
        tm_consensus_state.into(),
    )?;

    ctx.store_update_meta(
        client_id.clone(),
        client_state.latest_height,
        host_timestamp,
        host_height,
    )?;

    Ok(())
}

/// Update the host store with a new client state, pruning old states from the
/// store if need be.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateExecution`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn update_state<E>(
    client_state: &ClientStateType,
    ctx: &mut E,
    client_id: &ClientId,
    header: Any,
) -> Result<Vec<Height>, ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<E::ConsensusStateRef>>::Error: Into<ClientError>,
{
    let header = TmHeader::try_from(header)?;
    let header_height = header.height();

    prune_oldest_consensus_state(client_state, ctx, client_id)?;

    let maybe_existing_consensus_state = {
        let path_at_header_height = ClientConsensusStatePath::new(
            client_id.clone(),
            header_height.revision_number(),
            header_height.revision_height(),
        );

        ctx.consensus_state(&path_at_header_height).ok()
    };

    if maybe_existing_consensus_state.is_some() {
        // if we already had the header installed by a previous relayer
        // then this is a no-op.
        //
        // Do nothing.
    } else {
        let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
        let host_height = ExtClientValidationContext::host_height(ctx)?;

        let new_consensus_state = ConsensusStateType::from(header.clone());
        let new_client_state = client_state.clone().with_header(header)?;

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                header_height.revision_number(),
                header_height.revision_height(),
            ),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            new_client_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            header_height,
            host_timestamp,
            host_height,
        )?;
    }

    Ok(vec![header_height])
}

/// Commit a frozen client state, which was frozen as a result of having exhibited
/// misbehaviour, to the store.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateExecution`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn update_on_misbehaviour<E>(
    client_state: &ClientStateType,
    ctx: &mut E,
    client_id: &ClientId,
    _client_message: Any,
) -> Result<(), ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
{
    // NOTE: frozen height is  set to `Height {revision_height: 0,
    // revision_number: 1}` and it is the same for all misbehaviour. This
    // aligns with the
    // [`ibc-go`](https://github.com/cosmos/ibc-go/blob/0e3f428e66d6fc0fc6b10d2f3c658aaa5000daf7/modules/light-clients/07-tendermint/misbehaviour.go#L18-L19)
    // implementation.
    let frozen_client_state = client_state.clone().with_frozen_height(Height::min(0));

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        frozen_client_state.into(),
    )?;

    Ok(())
}

/// Commit the new client state and consensus state to the store upon a
/// successful client upgrade.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateExecution`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn update_on_upgrade<E>(
    client_state: &ClientStateType,
    ctx: &mut E,
    client_id: &ClientId,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
) -> Result<Height, ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
{
    let mut upgraded_tm_client_state = ClientState::try_from(upgraded_client_state)?;
    let upgraded_tm_cons_state: ConsensusStateType = upgraded_consensus_state.try_into()?;

    upgraded_tm_client_state.0.zero_custom_fields();

    // Construct new client state and consensus state relayer chosen client
    // parameters are ignored. All chain-chosen parameters come from
    // committed client, all client-chosen parameters come from current
    // client.
    let new_client_state = ClientStateType::new(
        upgraded_tm_client_state.0.chain_id,
        client_state.trust_level,
        client_state.trusting_period,
        upgraded_tm_client_state.0.unbonding_period,
        client_state.max_clock_drift,
        upgraded_tm_client_state.0.latest_height,
        upgraded_tm_client_state.0.proof_specs,
        upgraded_tm_client_state.0.upgrade_path,
        client_state.allow_update,
    )?;

    // The new consensus state is merely used as a trusted kernel against
    // which headers on the new chain can be verified. The root is just a
    // stand-in sentinel value as it cannot be known in advance, thus no
    // proof verification will pass. The timestamp and the
    // NextValidatorsHash of the consensus state is the blocktime and
    // NextValidatorsHash of the last block committed by the old chain. This
    // will allow the first block of the new chain to be verified against
    // the last validators of the old chain so long as it is submitted
    // within the TrustingPeriod of this client.
    // NOTE: We do not set processed time for this consensus state since
    // this consensus state should not be used for packet verification as
    // the root is empty. The next consensus state submitted using update
    // will be usable for packet-verification.
    let sentinel_root = b"sentinel_root".to_vec();
    let new_consensus_state = ConsensusStateType::new(
        sentinel_root.into(),
        upgraded_tm_cons_state.timestamp(),
        upgraded_tm_cons_state.next_validators_hash,
    );

    let latest_height = new_client_state.latest_height;
    let host_timestamp = ExtClientValidationContext::host_timestamp(ctx)?;
    let host_height = ExtClientValidationContext::host_height(ctx)?;

    ctx.store_client_state(
        ClientStatePath::new(client_id.clone()),
        new_client_state.into(),
    )?;
    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            client_id.clone(),
            latest_height.revision_number(),
            latest_height.revision_height(),
        ),
        new_consensus_state.into(),
    )?;
    ctx.store_update_meta(
        client_id.clone(),
        latest_height,
        host_timestamp,
        host_height,
    )?;

    Ok(latest_height)
}

/// Removes consensus states from the client store whose timestamps
/// are less than or equal to the host timestamp. This ensures that
/// the client store does not amass a buildup of stale consensus states.
pub fn prune_oldest_consensus_state<E>(
    client_state: &ClientStateType,
    ctx: &mut E,
    client_id: &ClientId,
) -> Result<(), ClientError>
where
    E: ClientExecutionContext + ExtClientValidationContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<E::ConsensusStateRef>>::Error: Into<ClientError>,
{
    let mut heights = ctx.consensus_state_heights(client_id)?;

    heights.sort();

    for height in heights {
        let client_consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        let consensus_state = ctx.consensus_state(&client_consensus_state_path)?;
        let tm_consensus_state: ConsensusStateType =
            consensus_state.try_into().map_err(Into::into)?;

        let host_timestamp =
            ctx.host_timestamp()?
                .into_tm_time()
                .ok_or_else(|| ClientError::Other {
                    description: String::from("host timestamp is not a valid TM timestamp"),
                })?;

        let tm_consensus_state_timestamp = tm_consensus_state.timestamp();
        let tm_consensus_state_expiry = (tm_consensus_state_timestamp
            + client_state.trusting_period)
            .map_err(|_| ClientError::Other {
                description: String::from(
                    "Timestamp overflow error occurred while attempting to parse TmConsensusState",
                ),
            })?;

        if tm_consensus_state_expiry > host_timestamp {
            break;
        }

        ctx.delete_consensus_state(client_consensus_state_path)?;
        ctx.delete_update_meta(client_id.clone(), height)?;
    }

    Ok(())
}

/// Update the `client_state`'s ID, trusting period, latest height, processed height,
/// and processed time metadata values to those values provided by a verified substitute
/// client state in response to a successful client recovery.
///
/// Note that unlike the `update_on_upgrade` function, `update_on_recovery` assumes
/// that the client being updated has already been re-initialised such that its original
/// client and consensus states have been overwritten to their new states.
///
/// This function is typically implemented as part of the [`ClientStateExecution`]
/// trait, but has been made standalone in order to enable greater flexibility
/// of the ClientState APIs.
pub fn update_on_recovery<E>(
    subject_client_state: ClientStateType,
    ctx: &mut E,
    subject_client_id: &ClientId,
    substitute_client_state: Any,
    substitute_consensus_state: Any,
) -> Result<(), ClientError>
where
    E: ExtClientExecutionContext,
    E::ClientStateRef: From<ClientStateType>,
    ConsensusStateType: Convertible<E::ConsensusStateRef>,
{
    let substitute_client_state = ClientState::try_from(substitute_client_state)?
        .inner()
        .clone();

    let chain_id = substitute_client_state.chain_id;
    let trusting_period = substitute_client_state.trusting_period;
    let latest_height = substitute_client_state.latest_height;

    let new_client_state = ClientStateType {
        chain_id,
        trusting_period,
        latest_height,
        frozen_height: None,
        ..subject_client_state
    };

    let host_timestamp = E::host_timestamp(ctx)?;
    let host_height = E::host_height(ctx)?;

    let tm_consensus_state: ConsensusStateType = substitute_consensus_state.try_into()?;

    ctx.store_consensus_state(
        ClientConsensusStatePath::new(
            subject_client_id.clone(),
            new_client_state.latest_height.revision_number(),
            new_client_state.latest_height.revision_height(),
        ),
        tm_consensus_state.into(),
    )?;

    ctx.store_client_state(
        ClientStatePath::new(subject_client_id.clone()),
        new_client_state.into(),
    )?;

    ctx.store_update_meta(
        subject_client_id.clone(),
        latest_height,
        host_timestamp,
        host_height,
    )?;

    Ok(())
}
