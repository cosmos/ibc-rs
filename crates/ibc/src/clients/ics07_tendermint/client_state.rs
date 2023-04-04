use crate::core::ics02_client::msgs::update_client::UpdateClientKind;
use crate::prelude::*;

use core::cmp::max;
use core::convert::{TryFrom, TryInto};
use core::time::Duration;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::commitment::v1::{MerklePath, MerkleProof as RawMerkleProof};
use ibc_proto::ibc::lightclients::tendermint::v1::{
    ClientState as RawTmClientState, ConsensusState as RawTmConsensusState,
};
use ibc_proto::protobuf::Protobuf;
use prost::Message;
use tendermint::chain::id::MAX_LENGTH as MaxChainIdLen;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};
use tendermint_light_client_verifier::{ProdVerifier, Verifier};

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use crate::clients::ics07_tendermint::error::{Error, IntoResult};
use crate::clients::ics07_tendermint::header::{Header as TmHeader, Header};
use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
use crate::core::ics02_client::client_state::{ClientState as Ics2ClientState, UpdatedState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::trust_threshold::TrustThreshold;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics23_commitment::merkle::{apply_prefix, MerkleProof};
use crate::core::ics23_commitment::specs::ProofSpecs;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath, ClientUpgradePath};
use crate::core::ics24_host::Path;
use crate::timestamp::{Timestamp, ZERO_DURATION};
use crate::Height;

use super::client_type as tm_client_type;

use crate::core::context::ContextError;

use crate::core::{ExecutionContext, ValidationContext};

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientState {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    max_clock_drift: Duration,
    latest_height: Height,
    pub proof_specs: ProofSpecs,
    pub upgrade_path: Vec<String>,
    allow_update: AllowUpdate,
    frozen_height: Option<Height>,
    #[cfg_attr(feature = "serde", serde(skip))]
    verifier: ProdVerifier,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllowUpdate {
    pub after_expiry: bool,
    pub after_misbehaviour: bool,
}

impl ClientState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
        frozen_height: Option<Height>,
    ) -> Result<ClientState, Error> {
        if chain_id.as_str().len() > MaxChainIdLen {
            return Err(Error::ChainIdTooLong {
                chain_id: chain_id.clone(),
                len: chain_id.as_str().len(),
                max_len: MaxChainIdLen,
            });
        }

        // `TrustThreshold` is guaranteed to be in the range `[0, 1)`, but a `TrustThreshold::ZERO`
        // value is invalid in this context
        if trust_level == TrustThreshold::ZERO {
            return Err(Error::InvalidTrustThreshold {
                reason: "ClientState trust-level cannot be zero".to_string(),
            });
        }

        let _ = TendermintTrustThresholdFraction::new(
            trust_level.numerator(),
            trust_level.denominator(),
        )
        .map_err(Error::InvalidTendermintTrustThreshold)?;

        // Basic validation of trusting period and unbonding period: each should be non-zero.
        if trusting_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState trusting period ({trusting_period:?}) must be greater than zero"
                ),
            });
        }

        if unbonding_period <= Duration::new(0, 0) {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                    "ClientState unbonding period ({unbonding_period:?}) must be greater than zero"
                ),
            });
        }

        if trusting_period >= unbonding_period {
            return Err(Error::InvalidTrustThreshold {
                reason: format!(
                "ClientState trusting period ({trusting_period:?}) must be smaller than unbonding period ({unbonding_period:?})"
            ),
            });
        }

        if max_clock_drift <= Duration::new(0, 0) {
            return Err(Error::InvalidMaxClockDrift {
                reason: "ClientState max-clock-drift must be greater than zero".to_string(),
            });
        }

        if latest_height.revision_number() != chain_id.version() {
            return Err(Error::InvalidLatestHeight {
                reason: "ClientState latest-height revision number must match chain-id version"
                    .to_string(),
            });
        }

        // Disallow empty proof-specs
        if proof_specs.is_empty() {
            return Err(Error::Validation {
                reason: "ClientState proof-specs cannot be empty".to_string(),
            });
        }

        // `upgrade_path` itself may be empty, but if not then each key must be non-empty
        for (idx, key) in upgrade_path.iter().enumerate() {
            if key.trim().is_empty() {
                return Err(Error::Validation {
                    reason: format!(
                        "ClientState upgrade-path key at index {idx:?} cannot be empty"
                    ),
                });
            }
        }

        Ok(Self {
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            proof_specs,
            upgrade_path,
            allow_update,
            frozen_height,
            verifier: ProdVerifier::default(),
        })
    }

    pub fn with_header(self, header: TmHeader) -> Result<Self, Error> {
        Ok(ClientState {
            latest_height: max(header.height(), self.latest_height),
            ..self
        })
    }

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    /// Get the refresh time to ensure the state does not expire
    pub fn refresh_time(&self) -> Option<Duration> {
        Some(2 * self.trusting_period / 3)
    }

    /// Helper method to produce a [`Options`] struct for use in
    /// Tendermint-specific light client verification.
    pub fn as_light_client_options(&self) -> Result<Options, Error> {
        Ok(Options {
            trust_threshold: self.trust_level.try_into().map_err(|e: ClientError| {
                Error::InvalidTrustThreshold {
                    reason: e.to_string(),
                }
            })?,
            trusting_period: self.trusting_period,
            clock_drift: self.max_clock_drift,
        })
    }

    fn verify_header(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        header: TmHeader,
    ) -> Result<(), ClientError> {
        // The tendermint-light-client crate though works on heights that are assumed
        // to have the same revision number. We ensure this here.
        if header.height().revision_number() != self.chain_id().version() {
            return Err(ClientError::ClientSpecific {
                description: Error::MismatchedRevisions {
                    current_revision: self.chain_id().version(),
                    update_revision: header.height().revision_number(),
                }
                .to_string(),
            });
        }

        // We also need to ensure that the trusted height (representing the
        // height of the header already on chain for which this client update is
        // based on) must be smaller than height of the new header that we're
        // installing.
        if header.height() <= header.trusted_height {
            return Err(ClientError::HeaderVerificationFailure {
                reason: format!(
                    "header height <= header trusted height ({} <= {})",
                    header.height(),
                    header.trusted_height
                ),
            });
        }

        // Delegate to tendermint-light-client, which contains the required checks
        // of the new header against the trusted consensus state.
        {
            let trusted_state =
                {
                    let trusted_client_cons_state_path =
                        ClientConsensusStatePath::new(&client_id, &header.trusted_height);
                    let trusted_consensus_state = downcast_tm_consensus_state(
                        ctx.consensus_state(&trusted_client_cons_state_path)?
                            .as_ref(),
                    )?;

                    check_header_trusted_next_validator_set(&header, &trusted_consensus_state)?;

                    TrustedBlockState {
                        chain_id: &self.chain_id.clone().into(),
                        header_time: trusted_consensus_state.timestamp,
                        height: header.trusted_height.revision_height().try_into().map_err(
                            |_| ClientError::ClientSpecific {
                                description: Error::InvalidHeaderHeight {
                                    height: header.trusted_height.revision_height(),
                                }
                                .to_string(),
                            },
                        )?,
                        next_validators: &header.trusted_next_validator_set,
                        next_validators_hash: trusted_consensus_state.next_validators_hash,
                    }
                };

            let untrusted_state = UntrustedBlockState {
                signed_header: &header.signed_header,
                validators: &header.validator_set,
                // NB: This will skip the
                // VerificationPredicates::next_validators_match check for the
                // untrusted state.
                next_validators: None,
            };

            let options = self.as_light_client_options()?;
            let now = ctx.host_timestamp()?.into_tm_time().unwrap();

            // main header verification, delegated to the tendermint-light-client crate.
            self.verifier
                .verify(untrusted_state, trusted_state, &options, now)
                .into_result()?;
        }

        Ok(())
    }

    // verify_misbehaviour determines whether or not two conflicting headers at
    // the same height would have convinced the light client.
    fn verify_misbehaviour(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        misbehaviour: TmMisbehaviour,
    ) -> Result<(), ClientError> {
        let header_1 = misbehaviour.header1();
        let trusted_consensus_state_1 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(&client_id, &header_1.trusted_height);
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            downcast_tm_consensus_state(consensus_state.as_ref())
        }?;

        let header_2 = misbehaviour.header2();
        let trusted_consensus_state_2 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(&client_id, &header_2.trusted_height);
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            downcast_tm_consensus_state(consensus_state.as_ref())
        }?;

        let current_timestamp = ctx.host_timestamp()?;

        self.check_misbehaviour_header(header_1, &trusted_consensus_state_1, current_timestamp)?;
        self.check_misbehaviour_header(header_2, &trusted_consensus_state_2, current_timestamp)
    }

    fn check_misbehaviour_header(
        &self,
        header: &TmHeader,
        trusted_consensus_state: &TmConsensusState,
        current_timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        // ensure correctness of the trusted next validator set provided by the relayer
        check_header_trusted_next_validator_set(header, trusted_consensus_state)?;

        // ensure header timestamp is within trusted period from the trusted consensus state
        {
            let duration_since_consensus_state = current_timestamp
                .duration_since(&trusted_consensus_state.timestamp())
                .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
                    time1: trusted_consensus_state.timestamp(),
                    time2: current_timestamp,
                })?;

            if duration_since_consensus_state >= self.trusting_period {
                return Err(Error::ConsensusStateTimestampGteTrustingPeriod {
                    duration_since_consensus_state,
                    trusting_period: self.trusting_period,
                }
                .into());
            }
        }

        // ensure that 2/3 of trusted validators have signed the new header
        {
            let untrusted_state = header.as_untrusted_block_state();
            let chain_id = self
                .chain_id
                .clone()
                .with_version(header.height().revision_number())
                .into();
            let trusted_state =
                Header::as_trusted_block_state(header, trusted_consensus_state, &chain_id)?;
            let options = self.as_light_client_options()?;

            self.verifier
                .verify_commit_against_trusted(&untrusted_state, &trusted_state, &options)
                .into_result()?;
        }

        // run the verification checks on the header based on the trusted
        // consensus state
        {
            let untrusted_state = header.as_untrusted_block_state();
            let chain_id = self.chain_id.clone().into();
            let trusted_state =
                header.as_trusted_block_state(trusted_consensus_state, &chain_id)?;
            let options = self.as_light_client_options()?;

            self.verifier
                .validate_against_trusted(
                    &untrusted_state,
                    &trusted_state,
                    &options,
                    current_timestamp.into_tm_time().unwrap(),
                )
                .into_result()?;
        }

        Ok(())
    }

    fn check_header_and_validator_set(
        &self,
        header: &Header,
        consensus_state: &TmConsensusState,
        current_timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        check_header_trusted_next_validator_set(header, consensus_state)?;

        let duration_since_consensus_state = current_timestamp
            .duration_since(&consensus_state.timestamp())
            .ok_or_else(|| ClientError::InvalidConsensusStateTimestamp {
                time1: consensus_state.timestamp(),
                time2: current_timestamp,
            })?;

        if duration_since_consensus_state >= self.trusting_period {
            return Err(Error::ConsensusStateTimestampGteTrustingPeriod {
                duration_since_consensus_state,
                trusting_period: self.trusting_period,
            }
            .into());
        }

        let untrusted_state = header.as_untrusted_block_state();
        let chain_id = self.chain_id.clone().into();
        let trusted_state = header.as_trusted_block_state(consensus_state, &chain_id)?;
        let options = self.as_light_client_options()?;

        self.verifier
            .validate_against_trusted(
                &untrusted_state,
                &trusted_state,
                &options,
                current_timestamp.into_tm_time().unwrap(),
            )
            .into_result()?;

        Ok(())
    }

    fn verify_header_commit_against_trusted(
        &self,
        header: &Header,
        consensus_state: &TmConsensusState,
    ) -> Result<(), ClientError> {
        let untrusted_state = header.as_untrusted_block_state();
        let chain_id = self.chain_id.clone().into();
        let trusted_state = Header::as_trusted_block_state(header, consensus_state, &chain_id)?;
        let options = self.as_light_client_options()?;

        self.verifier
            .verify_commit_against_trusted(&untrusted_state, &trusted_state, &options)
            .into_result()?;

        Ok(())
    }
}

impl Ics2ClientState for ClientState {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    fn client_type(&self) -> ClientType {
        tm_client_type()
    }

    fn latest_height(&self) -> Height {
        self.latest_height
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    fn confirm_not_frozen(&self) -> Result<(), ClientError> {
        if let Some(frozen_height) = self.frozen_height {
            return Err(ClientError::ClientFrozen {
                description: format!("the client is frozen at height {frozen_height}"),
            });
        }
        Ok(())
    }

    fn zero_custom_fields(&mut self) {
        // Reset custom fields to zero values
        self.trusting_period = ZERO_DURATION;
        self.trust_level = TrustThreshold::ZERO;
        self.allow_update.after_expiry = false;
        self.allow_update.after_misbehaviour = false;
        self.frozen_height = None;
        self.max_clock_drift = ZERO_DURATION;
    }

    fn expired(&self, elapsed: Duration) -> bool {
        elapsed > self.trusting_period
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError> {
        TmConsensusState::try_from(consensus_state).map(TmConsensusState::into_box)
    }

    fn verify_client_message(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        client_message: Any,
        update_kind: UpdateClientKind,
    ) -> Result<(), ClientError> {
        match update_kind {
            UpdateClientKind::UpdateHeader => {
                let header = TmHeader::try_from(client_message)?;
                self.verify_header(ctx, client_id, header)
            }
            UpdateClientKind::Misbehaviour => {
                let misbehaviour = TmMisbehaviour::try_from(client_message)?;
                self.verify_misbehaviour(ctx, client_id, misbehaviour)
            }
        }
    }

    // The misbehaviour checked for depends on the kind of message submitted:
    // + For a submitted `Header`, detects duplicate height misbehaviour and BFT time violation misbehaviour
    // + For a submitted `Misbehaviour`, verifies the correctness of the message
    fn check_for_misbehaviour(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        client_message: Any,
        update_kind: UpdateClientKind,
    ) -> Result<bool, ClientError> {
        match update_kind {
            UpdateClientKind::UpdateHeader => {
                let header = TmHeader::try_from(client_message)?;

                let header_consensus_state = TmConsensusState::from(header.clone());

                let maybe_existing_consensus_state = {
                    let path_at_header_height =
                        ClientConsensusStatePath::new(&client_id, &header.height());

                    ctx.consensus_state(&path_at_header_height).ok()
                };

                match maybe_existing_consensus_state {
                    Some(existing_consensus_state) => {
                        let existing_consensus_state =
                            downcast_tm_consensus_state(existing_consensus_state.as_ref())?;

                        // check if there is already a consensus state stored for the
                        // submitted header. If there is, there is no misbehaviour to report.
                        // Otherwise, we just confirmed that 2 headers exist with the same
                        // height, which is evidence of misbehaviour.
                        Ok(existing_consensus_state != header_consensus_state)
                    }
                    None => {
                        // If no header was previously installed, we ensure the monotonicity of timestamps.

                        // 1. for all headers, the new header needs to have a larger timestamp than
                        //    the “previous header”
                        {
                            let maybe_prev_cs =
                                ctx.prev_consensus_state(&client_id, &header.height())?;

                            if let Some(prev_cs) = maybe_prev_cs {
                                // New header timestamp cannot occur *before* the
                                // previous consensus state's height
                                let prev_cs = downcast_tm_consensus_state(prev_cs.as_ref())?;

                                if header.signed_header.header().time < prev_cs.timestamp {
                                    return Ok(true);
                                }
                            }
                        }

                        // 2. if a header comes in and is not the “last” header, then we also ensure
                        //    that its timestamp is less than the “next header”
                        if header.height() < self.latest_height() {
                            let maybe_next_cs =
                                ctx.next_consensus_state(&client_id, &header.height())?;

                            if let Some(next_cs) = maybe_next_cs {
                                // New (untrusted) header timestamp cannot occur *after* next
                                // consensus state's height
                                let next_cs = downcast_tm_consensus_state(next_cs.as_ref())?;

                                if header.signed_header.header().time > next_cs.timestamp {
                                    return Ok(true);
                                }
                            }
                        }

                        Ok(false)
                    }
                }
            }
            UpdateClientKind::Misbehaviour => {
                let misbehaviour = TmMisbehaviour::try_from(client_message)?;
                let header_1 = misbehaviour.header1();
                let header_2 = misbehaviour.header2();

                if header_1.height() == header_2.height() {
                    // when the height of the 2 headers are equal, we only have evidence
                    // of misbehaviour in the case where the headers are different
                    // (otherwise, the same header was added twice in the message,
                    // and this is evidence of nothing)
                    Ok(header_1.signed_header.commit.block_id.hash
                        != header_2.signed_header.commit.block_id.hash)
                } else {
                    // FIXME BEFORE MERGE: ibc-go ensures that header_1.height > header_2.height
                    // in Misbehaviour.ValidateBasic(). We are missing all these checks.

                    // header_1 is at greater height than header_2, therefore
                    // header_1 time must be less than or equal to
                    // header_2 time in order to be valid misbehaviour (violation of
                    // monotonic time).
                    Ok(header_1.signed_header.header.time <= header_2.signed_header.header.time)
                }
            }
        }
    }
    fn update_state(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: ClientId,
        header: Any,
    ) -> Result<(), ClientError> {
        let header = TmHeader::try_from(header)?;

        let maybe_existing_consensus_state = {
            let path_at_header_height = ClientConsensusStatePath::new(&client_id, &header.height());

            ctx.consensus_state(&path_at_header_height).ok()
        };

        if maybe_existing_consensus_state.is_some() {
            // if we already had the header installed by a previous relayer
            // then this is a no-op.
            Ok(())
        } else {
            let new_consensus_state = TmConsensusState::from(header.clone()).into_box();
            let new_client_state = self.clone().with_header(header)?.into_box();

            ctx.store_update_time(
                client_id.clone(),
                new_client_state.latest_height(),
                ctx.host_timestamp()?,
            )?;
            ctx.store_update_height(
                client_id.clone(),
                new_client_state.latest_height(),
                ctx.host_height()?,
            )?;

            ctx.store_consensus_state(
                ClientConsensusStatePath::new(&client_id, &new_client_state.latest_height()),
                new_consensus_state,
            )?;
            ctx.store_client_state(ClientStatePath::new(&client_id), new_client_state)?;

            Ok(())
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: ClientId,
        _misbehaviour: Any,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self
            .clone()
            .with_frozen_height(Height::new(0, 1).unwrap())
            .into_box();

        ctx.store_client_state(ClientStatePath::new(&client_id), frozen_client_state)?;

        Ok(())
    }

    fn check_misbehaviour_and_update_state(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        misbehaviour: Any,
    ) -> Result<Box<dyn Ics2ClientState>, ContextError> {
        let misbehaviour = TmMisbehaviour::try_from(misbehaviour)?;
        let header_1 = misbehaviour.header1();
        let header_2 = misbehaviour.header2();

        if header_1.height() == header_2.height() {
            // Fork
            if header_1.signed_header.commit.block_id.hash
                == header_2.signed_header.commit.block_id.hash
            {
                return Err(ContextError::ClientError(
                    Error::MisbehaviourHeadersBlockHashesEqual.into(),
                ));
            }
        } else {
            // BFT time violation
            if header_1.signed_header.header.time > header_2.signed_header.header.time {
                return Err(ContextError::ClientError(
                    Error::MisbehaviourHeadersNotAtSameHeight.into(),
                ));
            }
        }
        let client_cons_state_path_1 =
            ClientConsensusStatePath::new(&client_id, &header_1.trusted_height);
        let consensus_state_1 = {
            let cs = ctx.consensus_state(&client_cons_state_path_1)?;
            downcast_tm_consensus_state(cs.as_ref())
        }?;

        let client_cons_state_path_2 =
            ClientConsensusStatePath::new(&client_id, &header_2.trusted_height);
        let consensus_state_2 = {
            let cs = ctx.consensus_state(&client_cons_state_path_2)?;
            downcast_tm_consensus_state(cs.as_ref())
        }?;

        let chain_id = self
            .chain_id
            .clone()
            .with_version(header_1.height().revision_number());
        if !misbehaviour.chain_id_matches(&chain_id) {
            return Err(ContextError::ClientError(
                Error::MisbehaviourHeadersChainIdMismatch {
                    header_chain_id: header_1.signed_header.header.chain_id.to_string(),
                    chain_id: self.chain_id.to_string(),
                }
                .into(),
            ));
        }

        let current_timestamp = ctx.host_timestamp()?;

        self.check_header_and_validator_set(header_1, &consensus_state_1, current_timestamp)?;
        self.check_header_and_validator_set(header_2, &consensus_state_2, current_timestamp)?;

        self.verify_header_commit_against_trusted(header_1, &consensus_state_1)?;
        self.verify_header_commit_against_trusted(header_2, &consensus_state_2)?;

        let client_state = downcast_tm_client_state(self)?.clone();
        Ok(client_state
            .with_frozen_height(Height::new(0, 1).unwrap())
            .into_box())
    }

    fn check_header_and_update_state(
        &self,
        ctx: &dyn ValidationContext,
        client_id: ClientId,
        header: Any,
    ) -> Result<UpdatedState, ClientError> {
        fn maybe_consensus_state(
            ctx: &dyn ValidationContext,
            client_cons_state_path: &ClientConsensusStatePath,
        ) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
            match ctx.consensus_state(client_cons_state_path) {
                Ok(cs) => Ok(Some(cs)),
                Err(e) => match e {
                    ContextError::ClientError(ClientError::ConsensusStateNotFound {
                        client_id: _,
                        height: _,
                    }) => Ok(None),
                    ContextError::ClientError(e) => Err(e),
                    _ => Err(ClientError::Other {
                        description: e.to_string(),
                    }),
                },
            }
        }

        let client_state = downcast_tm_client_state(self)?.clone();
        let header = TmHeader::try_from(header)?;

        if header.height().revision_number() != client_state.chain_id().version() {
            return Err(ClientError::ClientSpecific {
                description: Error::MismatchedRevisions {
                    current_revision: client_state.chain_id().version(),
                    update_revision: header.height().revision_number(),
                }
                .to_string(),
            });
        }

        // Check if a consensus state is already installed; if so it should
        // match the untrusted header.
        let header_consensus_state = TmConsensusState::from(header.clone());
        let client_cons_state_path = ClientConsensusStatePath::new(&client_id, &header.height());
        let existing_consensus_state = match maybe_consensus_state(ctx, &client_cons_state_path)? {
            Some(cs) => {
                let cs = downcast_tm_consensus_state(cs.as_ref())?;
                // If this consensus state matches, skip verification
                // (optimization)
                if cs == header_consensus_state {
                    // Header is already installed and matches the incoming
                    // header (already verified)
                    return Ok(UpdatedState {
                        client_state: client_state.into_box(),
                        consensus_state: cs.into_box(),
                    });
                }
                Some(cs)
            }
            None => None,
        };

        let trusted_client_cons_state_path =
            ClientConsensusStatePath::new(&client_id, &header.trusted_height);
        let trusted_consensus_state = downcast_tm_consensus_state(
            ctx.consensus_state(&trusted_client_cons_state_path)?
                .as_ref(),
        )?;

        let trusted_state = TrustedBlockState {
            chain_id: &self.chain_id.clone().into(),
            header_time: trusted_consensus_state.timestamp,
            height: header
                .trusted_height
                .revision_height()
                .try_into()
                .map_err(|_| ClientError::ClientSpecific {
                    description: Error::InvalidHeaderHeight {
                        height: header.trusted_height.revision_height(),
                    }
                    .to_string(),
                })?,
            next_validators: &header.trusted_next_validator_set,
            next_validators_hash: trusted_consensus_state.next_validators_hash,
        };

        let untrusted_state = UntrustedBlockState {
            signed_header: &header.signed_header,
            validators: &header.validator_set,
            // NB: This will skip the
            // VerificationPredicates::next_validators_match check for the
            // untrusted state.
            next_validators: None,
        };

        let options = client_state.as_light_client_options()?;
        let now = ctx.host_timestamp()?.into_tm_time().unwrap();

        self.verifier
            .verify(untrusted_state, trusted_state, &options, now)
            .into_result()?;

        // If the header has verified, but its corresponding consensus state
        // differs from the existing consensus state for that height, freeze the
        // client and return the installed consensus state.
        if let Some(cs) = existing_consensus_state {
            if cs != header_consensus_state {
                return Ok(UpdatedState {
                    client_state: client_state.with_frozen_height(header.height()).into_box(),
                    consensus_state: cs.into_box(),
                });
            }
        }

        // Monotonicity checks for timestamps for in-the-middle updates
        // (cs-new, cs-next, cs-latest)
        if header.height() < client_state.latest_height() {
            let maybe_next_cs = ctx
                .next_consensus_state(&client_id, &header.height())?
                .as_ref()
                .map(|cs| downcast_tm_consensus_state(cs.as_ref()))
                .transpose()?;

            if let Some(next_cs) = maybe_next_cs {
                // New (untrusted) header timestamp cannot occur after next
                // consensus state's height
                if header.signed_header.header().time > next_cs.timestamp {
                    return Err(ClientError::ClientSpecific {
                        description: Error::HeaderTimestampTooHigh {
                            actual: header.signed_header.header().time.to_string(),
                            max: next_cs.timestamp.to_string(),
                        }
                        .to_string(),
                    });
                }
            }
        }

        // (cs-trusted, cs-prev, cs-new)
        if header.trusted_height < header.height() {
            let maybe_prev_cs = ctx
                .prev_consensus_state(&client_id, &header.height())?
                .as_ref()
                .map(|cs| downcast_tm_consensus_state(cs.as_ref()))
                .transpose()?;

            if let Some(prev_cs) = maybe_prev_cs {
                // New (untrusted) header timestamp cannot occur before the
                // previous consensus state's height
                if header.signed_header.header().time < prev_cs.timestamp {
                    return Err(ClientError::ClientSpecific {
                        description: Error::HeaderTimestampTooLow {
                            actual: header.signed_header.header().time.to_string(),
                            min: prev_cs.timestamp.to_string(),
                        }
                        .to_string(),
                    });
                }
            }
        }

        Ok(UpdatedState {
            client_state: client_state.with_header(header.clone())?.into_box(),
            consensus_state: TmConsensusState::from(header).into_box(),
        })
    }

    /// Perform client-specific verifications and check all data in the new
    /// client state to be the same across all valid Tendermint clients for the
    /// new chain.
    ///
    /// You can learn more about how to upgrade IBC-connected SDK chains in
    /// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
    /// guide
    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: RawMerkleProof,
        proof_upgrade_consensus_state: RawMerkleProof,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        // Make sure that the client type is of Tendermint type `ClientState`
        let mut upgraded_tm_client_state = TmClientState::try_from(upgraded_client_state)?;

        // Make sure that the consensus type is of Tendermint type `ConsensusState`
        let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;

        // Note: verification of proofs that unmarshalled correctly has been done
        // while decoding the proto message into a `MsgEnvelope` domain type
        let merkle_proof_upgrade_client = MerkleProof::from(proof_upgrade_client);
        let merkle_proof_upgrade_cons_state = MerkleProof::from(proof_upgrade_consensus_state);

        // Make sure the latest height of the current client is not greater then
        // the upgrade height This condition checks both the revision number and
        // the height
        if self.latest_height() >= upgraded_tm_client_state.latest_height() {
            return Err(ClientError::LowUpgradeHeight {
                upgraded_height: self.latest_height(),
                client_height: upgraded_tm_client_state.latest_height(),
            });
        }

        // Check to see if the upgrade path is set
        let mut upgrade_path = self.upgrade_path.clone();
        if upgrade_path.pop().is_none() {
            return Err(ClientError::ClientSpecific {
                description: "cannot upgrade client as no upgrade path has been set".to_string(),
            });
        };

        let last_height = self.latest_height().revision_height();

        // Construct the merkle path for the client state
        let mut client_upgrade_path = upgrade_path.clone();
        client_upgrade_path.push(ClientUpgradePath::UpgradedClientState(last_height).to_string());

        let client_upgrade_merkle_path = MerklePath {
            key_path: client_upgrade_path,
        };

        upgraded_tm_client_state.zero_custom_fields();
        let client_state_value =
            Protobuf::<RawTmClientState>::encode_vec(&upgraded_tm_client_state)
                .map_err(ClientError::Encode)?;

        // Verify the proof of the upgraded client state
        merkle_proof_upgrade_client
            .verify_membership(
                &self.proof_specs,
                root.clone().into(),
                client_upgrade_merkle_path,
                client_state_value,
                0,
            )
            .map_err(ClientError::Ics23Verification)?;

        // Construct the merkle path for the consensus state
        let mut cons_upgrade_path = upgrade_path;
        cons_upgrade_path
            .push(ClientUpgradePath::UpgradedClientConsensusState(last_height).to_string());
        let cons_upgrade_merkle_path = MerklePath {
            key_path: cons_upgrade_path,
        };

        let cons_state_value = Protobuf::<RawTmConsensusState>::encode_vec(&upgraded_tm_cons_state)
            .map_err(ClientError::Encode)?;

        // Verify the proof of the upgraded consensus state
        merkle_proof_upgrade_cons_state
            .verify_membership(
                &self.proof_specs,
                root.clone().into(),
                cons_upgrade_merkle_path,
                cons_state_value,
                0,
            )
            .map_err(ClientError::Ics23Verification)?;

        Ok(())
    }

    // Commit the new client state and consensus state to the store
    fn update_state_with_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<UpdatedState, ClientError> {
        let upgraded_tm_client_state = TmClientState::try_from(upgraded_client_state)?;
        let upgraded_tm_cons_state = TmConsensusState::try_from(upgraded_consensus_state)?;

        // Frozen height is set to None fo the new client state
        let new_frozen_height = None;

        // Construct new client state and consensus state relayer chosen client
        // parameters are ignored. All chain-chosen parameters come from
        // committed client, all client-chosen parameters come from current
        // client.
        let new_client_state = TmClientState::new(
            upgraded_tm_client_state.chain_id,
            self.trust_level,
            self.trusting_period,
            upgraded_tm_client_state.unbonding_period,
            self.max_clock_drift,
            upgraded_tm_client_state.latest_height,
            upgraded_tm_client_state.proof_specs,
            upgraded_tm_client_state.upgrade_path,
            self.allow_update,
            new_frozen_height,
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
        let sentinel_root = "sentinel_root".as_bytes().to_vec();
        let new_consensus_state = TmConsensusState::new(
            sentinel_root.into(),
            upgraded_tm_cons_state.timestamp,
            upgraded_tm_cons_state.next_validators_hash,
        );

        Ok(UpdatedState {
            client_state: new_client_state.into_box(),
            consensus_state: new_consensus_state.into_box(),
        })
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let client_state = downcast_tm_client_state(self)?;

        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof: MerkleProof = RawMerkleProof::try_from(proof.clone())
            .map_err(ClientError::InvalidCommitmentProof)?
            .into();

        merkle_proof
            .verify_membership(
                &client_state.proof_specs,
                root.clone().into(),
                merkle_path,
                value,
                0,
            )
            .map_err(ClientError::Ics23Verification)
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        let client_state = downcast_tm_client_state(self)?;

        let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
        let merkle_proof: MerkleProof = RawMerkleProof::try_from(proof.clone())
            .map_err(ClientError::InvalidCommitmentProof)?
            .into();

        merkle_proof
            .verify_non_membership(&client_state.proof_specs, root.clone().into(), merkle_path)
            .map_err(ClientError::Ics23Verification)
    }
}

// `header.trusted_validator_set` was given to us by the relayer. Thus, we
// need to ensure that the relayer gave us the right set, i.e. by ensuring
// that it matches the hash we have stored on chain.
fn check_header_trusted_next_validator_set(
    header: &TmHeader,
    trusted_consensus_state: &TmConsensusState,
) -> Result<(), ClientError> {
    if header.trusted_next_validator_set.hash() == trusted_consensus_state.next_validators_hash {
        Ok(())
    } else {
        Err(ClientError::HeaderVerificationFailure {
            reason: "header trusted next validator set hash does not match hash stored on chain"
                .to_string(),
        })
    }
}

fn downcast_tm_client_state(cs: &dyn Ics2ClientState) -> Result<&ClientState, ClientError> {
    cs.as_any()
        .downcast_ref::<ClientState>()
        .ok_or_else(|| ClientError::ClientArgsTypeMismatch {
            client_type: tm_client_type(),
        })
}

fn downcast_tm_consensus_state(cs: &dyn ConsensusState) -> Result<TmConsensusState, ClientError> {
    cs.as_any()
        .downcast_ref::<TmConsensusState>()
        .ok_or_else(|| ClientError::ClientArgsTypeMismatch {
            client_type: tm_client_type(),
        })
        .map(Clone::clone)
}

impl Protobuf<RawTmClientState> for ClientState {}

impl TryFrom<RawTmClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawTmClientState) -> Result<Self, Self::Error> {
        let chain_id = ChainId::from_string(raw.chain_id.as_str());

        let trust_level = {
            let trust_level = raw
                .trust_level
                .clone()
                .ok_or(Error::MissingTrustingPeriod)?;
            trust_level
                .try_into()
                .map_err(|e| Error::InvalidTrustThreshold {
                    reason: format!("{e}"),
                })?
        };

        let trusting_period = raw
            .trusting_period
            .ok_or(Error::MissingTrustingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingTrustingPeriod)?;

        let unbonding_period = raw
            .unbonding_period
            .ok_or(Error::MissingUnbondingPeriod)?
            .try_into()
            .map_err(|_| Error::MissingUnbondingPeriod)?;

        let max_clock_drift = raw
            .max_clock_drift
            .ok_or(Error::NegativeMaxClockDrift)?
            .try_into()
            .map_err(|_| Error::NegativeMaxClockDrift)?;

        let latest_height = raw
            .latest_height
            .ok_or(Error::MissingLatestHeight)?
            .try_into()
            .map_err(|_| Error::MissingLatestHeight)?;

        // In `RawClientState`, a `frozen_height` of `0` means "not frozen".
        // See:
        // https://github.com/cosmos/ibc-go/blob/8422d0c4c35ef970539466c5bdec1cd27369bab3/modules/light-clients/07-tendermint/types/client_state.go#L74
        let frozen_height = raw
            .frozen_height
            .and_then(|raw_height| raw_height.try_into().ok());

        // We use set this deprecated field just so that we can properly convert
        // it back in its raw form
        #[allow(deprecated)]
        let allow_update = AllowUpdate {
            after_expiry: raw.allow_update_after_expiry,
            after_misbehaviour: raw.allow_update_after_misbehaviour,
        };

        let client_state = ClientState::new(
            chain_id,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            latest_height,
            raw.proof_specs.into(),
            raw.upgrade_path,
            allow_update,
            frozen_height,
        )?;

        Ok(client_state)
    }
}

impl From<ClientState> for RawTmClientState {
    fn from(value: ClientState) -> Self {
        #[allow(deprecated)]
        Self {
            chain_id: value.chain_id.to_string(),
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.map(|height| height.into()).unwrap_or(
                RawHeight {
                    revision_number: 0,
                    revision_height: 0,
                },
            )),
            latest_height: Some(value.latest_height.into()),
            proof_specs: value.proof_specs.into(),
            upgrade_path: value.upgrade_path,
            allow_update_after_expiry: value.allow_update.after_expiry,
            allow_update_after_misbehaviour: value.allow_update.after_misbehaviour,
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
            RawTmClientState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => {
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
            type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawTmClientState>::encode_vec(&client_state)
                .expect("encoding to `Any` from `TmClientState`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::Height;
    use core::time::Duration;
    use test_log::test;

    use ibc_proto::ics23::ProofSpec as Ics23ProofSpec;

    use crate::clients::ics07_tendermint::client_state::{
        AllowUpdate, ClientState as TmClientState,
    };
    use crate::core::ics02_client::client_state::ClientState;
    use crate::core::ics02_client::trust_threshold::TrustThreshold;
    use crate::core::ics23_commitment::specs::ProofSpecs;
    use crate::core::ics24_host::identifier::ChainId;
    use crate::timestamp::ZERO_DURATION;

    #[derive(Clone, Debug, PartialEq)]
    struct ClientStateParams {
        id: ChainId,
        trust_level: TrustThreshold,
        trusting_period: Duration,
        unbonding_period: Duration,
        max_clock_drift: Duration,
        latest_height: Height,
        proof_specs: ProofSpecs,
        upgrade_path: Vec<String>,
        allow_update: AllowUpdate,
    }

    #[test]
    fn client_state_new() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::default(),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(0, 10).unwrap(),
            proof_specs: ProofSpecs::default(),
            upgrade_path: Default::default(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            params: ClientStateParams,
            want_pass: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                name: "Valid parameters".to_string(),
                params: default_params.clone(),
                want_pass: true,
            },
            Test {
                name: "Valid (empty) upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec![],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid upgrade-path".to_string(),
                params: ClientStateParams {
                    upgrade_path: vec!["upgrade".to_owned(), "upgradedIBCState".to_owned()],
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Valid long (50 chars) chain-id".to_string(),
                params: ClientStateParams {
                    id: ChainId::new("a".repeat(48), 0),
                    ..default_params.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Invalid too-long (51 chars) chain-id".to_string(),
                params: ClientStateParams {
                    id: ChainId::new("a".repeat(49), 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) max-clock-drift period".to_string(),
                params: ClientStateParams {
                    max_clock_drift: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid unbonding period".to_string(),
                params: ClientStateParams {
                    unbonding_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting period".to_string(),
                params: ClientStateParams {
                    trusting_period: ZERO_DURATION,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too large) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(11, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (equal) trusting period w.r.t. unbonding period".to_string(),
                params: ClientStateParams {
                    trusting_period: Duration::new(10, 0),
                    unbonding_period: Duration::new(10, 0),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (zero) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::ZERO,
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (too small) trusting trust threshold".to_string(),
                params: ClientStateParams {
                    trust_level: TrustThreshold::new(1, 4).unwrap(),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid latest height revision number (doesn't match chain)".to_string(),
                params: ClientStateParams {
                    latest_height: Height::new(1, 1).unwrap(),
                    ..default_params.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Invalid (empty) proof specs".to_string(),
                params: ClientStateParams {
                    proof_specs: ProofSpecs::from(Vec::<Ics23ProofSpec>::new()),
                    ..default_params
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let cs_result = TmClientState::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
                None,
            );

            assert_eq!(
                test.want_pass,
                cs_result.is_ok(),
                "ClientState::new() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                cs_result.err(),
            );
        }
    }

    #[test]
    fn client_state_verify_height() {
        // Define a "default" set of parameters to reuse throughout these tests.
        let default_params: ClientStateParams = ClientStateParams {
            id: ChainId::new("ibc".to_string(), 1),
            trust_level: TrustThreshold::ONE_THIRD,
            trusting_period: Duration::new(64000, 0),
            unbonding_period: Duration::new(128000, 0),
            max_clock_drift: Duration::new(3, 0),
            latest_height: Height::new(1, 10).unwrap(),
            proof_specs: ProofSpecs::default(),
            upgrade_path: Default::default(),
            allow_update: AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
        };

        struct Test {
            name: String,
            height: Height,
            setup: Option<Box<dyn FnOnce(TmClientState) -> TmClientState>>,
            want_pass: bool,
        }

        let tests = vec![
            Test {
                name: "Successful height verification".to_string(),
                height: Height::new(1, 8).unwrap(),
                setup: None,
                want_pass: true,
            },
            Test {
                name: "Invalid (too large)  client height".to_string(),
                height: Height::new(1, 12).unwrap(),
                setup: None,
                want_pass: false,
            },
        ];

        for test in tests {
            let p = default_params.clone();
            let client_state = TmClientState::new(
                p.id,
                p.trust_level,
                p.trusting_period,
                p.unbonding_period,
                p.max_clock_drift,
                p.latest_height,
                p.proof_specs,
                p.upgrade_path,
                p.allow_update,
                None,
            )
            .unwrap();
            let client_state = match test.setup {
                Some(setup) => (setup)(client_state),
                _ => client_state,
            };
            let res = client_state.validate_proof_height(test.height);

            assert_eq!(
                test.want_pass,
                res.is_ok(),
                "ClientState::validate_proof_height() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.height,
                res.err(),
            );
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    use crate::test::test_serialization_roundtrip;

    #[test]
    fn serialization_roundtrip_no_proof() {
        let json_data =
            include_str!("../../../tests/support/query/serialization/client_state.json");
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }

    #[test]
    fn serialization_roundtrip_with_proof() {
        let json_data =
            include_str!("../../../tests/support/query/serialization/client_state_proof.json");
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }
}

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
    use crate::prelude::*;
    use core::time::Duration;

    use tendermint::block::Header;

    use crate::clients::ics07_tendermint::client_state::{AllowUpdate, ClientState};
    use crate::core::ics02_client::height::Height;
    use crate::core::ics24_host::identifier::ChainId;

    pub fn get_dummy_tendermint_client_state(tm_header: Header) -> ClientState {
        ClientState::new(
            ChainId::from(tm_header.chain_id.clone()),
            Default::default(),
            Duration::from_secs(64000),
            Duration::from_secs(128000),
            Duration::from_millis(3000),
            Height::new(
                ChainId::chain_version(tm_header.chain_id.as_str()),
                u64::from(tm_header.height),
            )
            .unwrap(),
            Default::default(),
            Default::default(),
            AllowUpdate {
                after_expiry: false,
                after_misbehaviour: false,
            },
            None,
        )
        .unwrap()
    }
}
