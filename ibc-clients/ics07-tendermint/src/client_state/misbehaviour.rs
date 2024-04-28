use ibc_client_tendermint_types::error::{Error, IntoResult};
use ibc_client_tendermint_types::{
    ConsensusState as ConsensusStateType, Header as TmHeader, Misbehaviour as TmMisbehaviour,
};
use ibc_core_client::context::{Convertible, ExtClientValidationContext};
use ibc_core_client::types::error::ClientError;
use ibc_core_host::types::identifiers::{ChainId, ClientId};
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;
use tendermint::crypto::Sha256;
use tendermint::merkle::MerkleHash;
use tendermint::{Hash, Time};
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::Verifier;

use crate::types::Header;

/// Determines whether or not two conflicting headers at the same height would
/// have convinced the light client.
pub fn verify_misbehaviour<V, H>(
    ctx: &V,
    misbehaviour: &TmMisbehaviour,
    client_id: &ClientId,
    chain_id: &ChainId,
    options: &Options,
    verifier: &impl Verifier,
) -> Result<(), ClientError>
where
    V: ExtClientValidationContext,
    ConsensusStateType: Convertible<V::ConsensusStateRef>,
    <ConsensusStateType as TryFrom<V::ConsensusStateRef>>::Error: Into<ClientError>,
    H: MerkleHash + Sha256 + Default,
{
    misbehaviour.validate_basic::<H>()?;

    let header_1 = misbehaviour.header1();
    let trusted_consensus_state_1: ConsensusStateType = {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            header_1.trusted_height.revision_number(),
            header_1.trusted_height.revision_height(),
        );
        let consensus_state = ctx.consensus_state(&consensus_state_path)?;

        consensus_state.try_into().map_err(Into::into)?
    };

    let header_2 = misbehaviour.header2();
    let trusted_consensus_state_2: ConsensusStateType = {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            header_2.trusted_height.revision_number(),
            header_2.trusted_height.revision_height(),
        );
        let consensus_state = ctx.consensus_state(&consensus_state_path)?;

        consensus_state.try_into().map_err(Into::into)?
    };

    let current_timestamp = ctx.host_timestamp()?;

    verify_misbehaviour_header::<H>(
        header_1,
        chain_id,
        options,
        trusted_consensus_state_1.timestamp(),
        trusted_consensus_state_1.next_validators_hash,
        current_timestamp,
        verifier,
    )?;
    verify_misbehaviour_header::<H>(
        header_2,
        chain_id,
        options,
        trusted_consensus_state_2.timestamp(),
        trusted_consensus_state_2.next_validators_hash,
        current_timestamp,
        verifier,
    )
}

pub fn verify_misbehaviour_header<H>(
    header: &TmHeader,
    chain_id: &ChainId,
    options: &Options,
    trusted_timestamp: Time,
    trusted_next_validator_hash: Hash,
    current_timestamp: Timestamp,
    verifier: &impl Verifier,
) -> Result<(), ClientError>
where
    H: MerkleHash + Sha256 + Default,
{
    // ensure correctness of the trusted next validator set provided by the relayer
    header.check_trusted_next_validator_set::<H>(&trusted_next_validator_hash)?;

    // ensure trusted consensus state is within trusting period
    {
        let duration_since_consensus_state = current_timestamp
            .duration_since(&trusted_timestamp.into())
            .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
                time1: trusted_timestamp.into(),
                time2: current_timestamp,
            })?;

        if duration_since_consensus_state >= options.trusting_period {
            return Err(Error::ConsensusStateTimestampGteTrustingPeriod {
                duration_since_consensus_state,
                trusting_period: options.trusting_period,
            }
            .into());
        }
    }

    // main header verification, delegated to the tendermint-light-client crate.
    let untrusted_state = header.as_untrusted_block_state();

    let tm_chain_id = &chain_id
        .as_str()
        .try_into()
        .map_err(|e| ClientError::Other {
            description: format!("failed to parse chain id: {e}"),
        })?;

    let trusted_state = header.as_trusted_block_state(
        tm_chain_id,
        trusted_timestamp,
        trusted_next_validator_hash,
    )?;

    let current_timestamp = current_timestamp.into_tm_time().ok_or(ClientError::Other {
        description: "host timestamp must not be zero".to_string(),
    })?;

    verifier
        .verify_misbehaviour_header(untrusted_state, trusted_state, options, current_timestamp)
        .into_result()?;

    Ok(())
}

pub fn check_for_misbehaviour_on_misbehavior(
    header_1: &Header,
    header_2: &Header,
) -> Result<bool, ClientError> {
    if header_1.height() == header_2.height() {
        // when the height of the 2 headers are equal, we only have evidence
        // of misbehaviour in the case where the headers are different
        // (otherwise, the same header was added twice in the message,
        // and this is evidence of nothing)
        Ok(header_1.signed_header.commit.block_id.hash
            != header_2.signed_header.commit.block_id.hash)
    } else {
        // header_1 is at greater height than header_2, therefore
        // header_1 time must be less than or equal to
        // header_2 time in order to be valid misbehaviour (violation of
        // monotonic time).
        Ok(header_1.signed_header.header.time <= header_2.signed_header.header.time)
    }
}
