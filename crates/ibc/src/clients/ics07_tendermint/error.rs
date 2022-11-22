use crate::prelude::*;

use crate::core::ics02_client::error::Error as Ics02Error;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::ClientId;
use crate::timestamp::{Timestamp, TimestampOverflowError};
use crate::Height;

use core::time::Duration;

use flex_error::{define_error, TraceError};
use tendermint::account::Id;
use tendermint::hash::Hash;
use tendermint::Error as TendermintError;
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;
use tendermint_light_client_verifier::operations::VotingPowerTally;
use tendermint_light_client_verifier::types::ValidatorSet;
use tendermint_light_client_verifier::Verdict;

define_error! {
    #[derive(Debug, PartialEq, Eq)]
    Error {
        ChainIdTooLong
            {
                chain_id: String,
                len: usize,
                max_len: usize,
            }
            |e| { format_args!("chain-id is ({0}) is too long, got: {1}, max allowed: {2}", e.chain_id, e.len, e.max_len) },

        InvalidTrustingPeriod
            { reason: String }
            |e| { format_args!("invalid trusting period: {}", e.reason) },

        InvalidUnbondingPeriod
            { reason: String }
            |e| { format_args!("invalid unbonding period: {}", e.reason) },

        InvalidAddress
            |_| { "invalid address" },

        InvalidHeader
            { reason: String }
            [ TendermintError ]
            |e| { format_args!("invalid header, failed basic validation: {}", e.reason) },

        InvalidTrustThreshold
            { reason: String }
            |e| { format_args!("invalid client state trust threshold: {}", e.reason) },

        InvalidTendermintTrustThreshold
            [ TendermintError ]
            |_| { "invalid tendermint client state trust threshold" },

        InvalidMaxClockDrift
            { reason: String }
            |e| { format_args!("invalid client state max clock drift: {}", e.reason) },

        InvalidLatestHeight
            { reason: String }
            |e| { format_args!("invalid client state latest height: {}", e.reason) },

        MissingSignedHeader
            |_| { "missing signed header" },

        Validation
            { reason: String }
            |e| { format_args!("invalid header, failed basic validation: {}", e.reason) },

        InvalidRawClientState
            { reason: String }
            |e| { format_args!("invalid raw client state: {}", e.reason) },

        InvalidRawClientId
            { client_id: String }
            |e| { format_args!("invalid raw client id: {}", e.client_id) },

        MissingValidatorSet
            |_| { "missing validator set" },

        MissingTrustedValidatorSet
            |_| { "missing trusted validator set" },

        MissingTrustedHeight
            |_| { "missing trusted height" },

        MissingTrustingPeriod
            |_| { "missing trusting period" },

        MissingUnbondingPeriod
            |_| { "missing unbonding period" },

        InvalidChainIdentifier
            [ ValidationError ]
            |_| { "invalid chain identifier" },

        NegativeTrustingPeriod
            |_| { "negative trusting period" },

        NegativeUnbondingPeriod
            |_| { "negative unbonding period" },

        MissingMaxClockDrift
            |_| { "missing max clock drift" },

        NegativeMaxClockDrift
            |_| {  "negative max clock drift" },

        MissingLatestHeight
            |_| { "missing latest height" },

        InvalidFrozenHeight
            |_| { "invalid frozen height" },

        InvalidChainId
            { raw_value: String }
            [ ValidationError ]
            |e| { format_args!("invalid chain identifier: {}", e.raw_value) },

        InvalidRawHeight
            { raw_height: u64 }
            |e| { format_args!("invalid raw height: {}", e.raw_height) },

        InvalidRawConsensusState
            { reason: String }
            | e | { format_args!("invalid raw client consensus state: {}", e.reason) },

        InvalidRawHeader
            [ TendermintError ]
            | _ | { "invalid raw header" },

        InvalidRawMisbehaviour
            { reason: String }
            | e | { format_args!("invalid raw misbehaviour: {}", e.reason) },

        Decode
            [ TraceError<prost::DecodeError> ]
            | _ | { "decode error" },

        InsufficientVotingPower
            { reason: String }
            | e | {
                format_args!("insufficient overlap: {}", e.reason)
            },

        LowUpdateTimestamp
            {
                low: String,
                high: String
            }
            | e | {
                format_args!("header timestamp {0} must be greater than current client consensus state timestamp {1}", e.low, e.high)
            },

        HeaderTimestampOutsideTrustingTime
            {
                low: String,
                high: String
            }
            | e | {
                format_args!("header timestamp {0} is outside the trusting period w.r.t. consensus state timestamp {1}", e.low, e.high)
            },

        HeaderTimestampTooHigh
            {
                actual: String,
                max: String,
            }
            | e | {
                format_args!("given other previous updates, header timestamp should be at most {0}, but was {1}", e.max, e.actual)
            },

        HeaderTimestampTooLow
            {
                actual: String,
                min: String,
            }
            | e | {
                format_args!("given other previous updates, header timestamp should be at least {0}, but was {1}", e.min, e.actual)
            },

        TimestampOverflow
            [ TimestampOverflowError ]
            |_| { "timestamp overflowed" },

        NotEnoughTimeElapsed
            {
                current_time: Timestamp,
                earliest_time: Timestamp,
            }
            | e | {
                format_args!("not enough time elapsed, current timestamp {0} is still less than earliest acceptable timestamp {1}", e.current_time, e.earliest_time)
            },

        NotEnoughBlocksElapsed
            {
                current_height: Height,
                earliest_height: Height,
            }
            | e | {
                format_args!("not enough blocks elapsed, current height {0} is still less than earliest acceptable height {1}", e.current_height, e.earliest_height)
            },

        InvalidHeaderHeight
            { height: u64 }
            | e | {
                format_args!("header revision height = {0} is invalid", e.height)
            },

        InvalidTrustedHeaderHeight
            {
                trusted_header_height: Height,
                height_header: Height
            }
            | e | {
                format_args!("header height is {0} and is lower than the trusted header height, which is {1} ", e.height_header, e.trusted_header_height)
            },

        LowUpdateHeight
            {
                low: Height,
                high: Height
            }
            | e | {
                format_args!("header height is {0} but it must be greater than the current client height which is {1}", e.low, e.high)
            },

        MismatchedRevisions
            {
                current_revision: u64,
                update_revision: u64,
            }
            | e | {
                format_args!("the header's current/trusted revision number ({0}) and the update's revision number ({1}) should be the same", e.current_revision, e.update_revision)
            },

        InvalidValidatorSet
            {
                hash1: Hash,
                hash2: Hash,
            }
            | e | {
                format_args!("invalid validator set: header_validators_hash={} and validators_hash={}", e.hash1, e.hash2)
            },

        NotEnoughTrustedValsSigned
            { tally: VotingPowerTally }
            | e | {
                format_args!("not enough trust because insufficient validators overlap: {:?}", e.tally)
            },

        VerificationError
            { detail: LightClientErrorDetail }
            | e | {
                format_args!("verification failed: {}", e.detail)
            },

        ProcessedTimeNotFound
            {
                client_id: ClientId,
                height: Height,
            }
            | e | {
                format_args!(
                    "Processed time for the client {0} at height {1} not found",
                    e.client_id, e.height)
            },

        ProcessedHeightNotFound
            {
                client_id: ClientId,
                height: Height,
            }
            | e | {
                format_args!(
                    "Processed height for the client {0} at height {1} not found",
                    e.client_id, e.height)
            },

        InsufficientHeight
            {
                latest_height: Height,
                target_height: Height,
            }
            | e | {
                format_args!("the height is insufficient: latest_height={0} target_height={1}", e.latest_height, e.target_height)
            },

        ClientFrozen
            {
                frozen_height: Height,
                target_height: Height,
            }
            | e | {
                format_args!("the client is frozen: frozen_height={0} target_height={1}", e.frozen_height, e.target_height)
            },

        MisbehaviourHeadersChainIdMismatch {
                header_chain_id: String,
                chain_id: String,
            }
            | e | { format_args!("header chain-id ('{0}') does not match the light client's chain-id ('{1}')", e.header_chain_id, e.chain_id) },

        MisbehaviourHeadersBlockHashesEqual
            |_| { "headers block hashes are equal" },

        MisbehaviourHeadersNotAtSameHeight
            |_| { "headers are not at same height and are monotonically increasing" },

        MisbehaviourTrustedValidatorHashMismatch
            {
                trusted_validator_set: ValidatorSet,
                next_validators_hash: Hash,
                trusted_val_hash: Hash,
            }
            | e | {
                format_args!("trusted validators {:?}, does not hash to latest trusted validators. Expected: {:?}, got: {:?}", e.trusted_validator_set, e.next_validators_hash, e.trusted_val_hash)
            },

        MisbehaviourConsensusStateTimestampGteTrustingPeriod
            {
                duration_since_consensus_state: Duration,
                trusting_period: Duration,
            }
            | e | {
                format_args!("current timestamp minus the latest consensus state timestamp is greater than or equal to the trusting period ({:?} >= {:?})", e.duration_since_consensus_state, e.trusting_period)
            },
    }
}

define_error! {
    #[derive(Debug, PartialEq, Eq)]
    VerificationError {
        InvalidSignature
            | _ | { "couldn't verify validator signature" },

        DuplicateValidator
            { id: Id }
            | e | {
                format_args!("duplicate validator in commit signatures with address {}", e.id)
            },

        InsufficientOverlap
            { q1: u64, q2: u64 }
            | e | {
                format_args!("insufficient signers overlap between {0} and {1}", e.q1, e.q2)
            },
    }
}

impl From<Error> for Ics02Error {
    fn from(e: Error) -> Self {
        Self::client_specific(e.to_string())
    }
}

pub(crate) trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl IntoResult<(), Error> for Verdict {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Verdict::Success => Ok(()),
            Verdict::NotEnoughTrust(tally) => Err(Error::not_enough_trusted_vals_signed(tally)),
            Verdict::Invalid(error) => Err(Error::verification_error(error)),
        }
    }
}
