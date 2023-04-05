use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::prelude::*;

use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use crate::clients::ics07_tendermint::error::{Error, IntoResult};
use crate::clients::ics07_tendermint::header::Header as TmHeader;
use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::{ics24_host::identifier::ClientId, ValidationContext};
use crate::timestamp::Timestamp;

use super::{check_header_trusted_next_validator_set, downcast_tm_consensus_state, ClientState};

impl ClientState {
    // verify_misbehaviour determines whether or not two conflicting headers at
    // the same height would have convinced the light client.
    pub fn verify_misbehaviour(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        misbehaviour: TmMisbehaviour,
    ) -> Result<(), ClientError> {
        let header_1 = misbehaviour.header1();
        let trusted_consensus_state_1 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(client_id, &header_1.trusted_height);
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            downcast_tm_consensus_state(consensus_state.as_ref())
        }?;

        let header_2 = misbehaviour.header2();
        let trusted_consensus_state_2 = {
            let consensus_state_path =
                ClientConsensusStatePath::new(client_id, &header_2.trusted_height);
            let consensus_state = ctx.consensus_state(&consensus_state_path)?;

            downcast_tm_consensus_state(consensus_state.as_ref())
        }?;

        self.check_misbehaviour_headers_consistency(header_1, header_2)?;

        let current_timestamp = ctx.host_timestamp()?;
        self.check_misbehaviour_header(header_1, &trusted_consensus_state_1, current_timestamp)?;
        self.check_misbehaviour_header(header_2, &trusted_consensus_state_2, current_timestamp)
    }

    pub fn check_misbehaviour_headers_consistency(
        &self,
        header_1: &TmHeader,
        header_2: &TmHeader,
    ) -> Result<(), ClientError> {
        if header_1.signed_header.header.chain_id != header_2.signed_header.header.chain_id {
            return Err(Error::InvalidRawMisbehaviour {
                reason: "headers must have identical chain_ids".to_owned(),
            }
            .into());
        }

        if header_1.height() < header_2.height() {
            return Err(Error::InvalidRawMisbehaviour {
                reason: format!(
                    "headers1 height is less than header2 height ({} < {})",
                    header_1.height(),
                    header_2.height()
                ),
            }
            .into());
        }

        Ok(())
    }

    pub fn check_misbehaviour_header(
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
                TmHeader::as_trusted_block_state(header, trusted_consensus_state, &chain_id)?;
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
}
