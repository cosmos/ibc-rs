use crate::prelude::*;

use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};
use tendermint_light_client_verifier::Verifier;

use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use crate::clients::ics07_tendermint::error::{Error, IntoResult};
use crate::clients::ics07_tendermint::header::Header as TmHeader;
use crate::core::ics02_client::client_state::ClientState as Ics2ClientState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::{ics24_host::identifier::ClientId, ValidationContext};

use super::{check_header_trusted_next_validator_set, downcast_tm_consensus_state, ClientState};

impl ClientState {
    pub fn verify_header(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
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
                        ClientConsensusStatePath::new(client_id, &header.trusted_height);
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

    pub fn check_for_misbehaviour_update_client(
        &self,
        ctx: &dyn ValidationContext,
        client_id: &ClientId,
        header: TmHeader,
    ) -> Result<bool, ClientError> {
        let header_consensus_state = TmConsensusState::from(header.clone());

        let maybe_existing_consensus_state = {
            let path_at_header_height = ClientConsensusStatePath::new(client_id, &header.height());

            ctx.consensus_state(&path_at_header_height).ok()
        };

        match maybe_existing_consensus_state {
            Some(existing_consensus_state) => {
                let existing_consensus_state =
                    downcast_tm_consensus_state(existing_consensus_state.as_ref())?;

                // There is evidence of misbehaviour if the stored consensus state
                // is different from the new one we received.
                Ok(existing_consensus_state != header_consensus_state)
            }
            None => {
                // If no header was previously installed, we ensure the monotonicity of timestamps.

                // 1. for all headers, the new header needs to have a larger timestamp than
                //    the “previous header”
                {
                    let maybe_prev_cs = ctx.prev_consensus_state(client_id, &header.height())?;

                    if let Some(prev_cs) = maybe_prev_cs {
                        // New header timestamp cannot occur *before* the
                        // previous consensus state's height
                        let prev_cs = downcast_tm_consensus_state(prev_cs.as_ref())?;

                        if header.signed_header.header().time <= prev_cs.timestamp {
                            return Ok(true);
                        }
                    }
                }

                // 2. if a header comes in and is not the “last” header, then we also ensure
                //    that its timestamp is less than the “next header”
                if header.height() < self.latest_height() {
                    let maybe_next_cs = ctx.next_consensus_state(client_id, &header.height())?;

                    if let Some(next_cs) = maybe_next_cs {
                        // New (untrusted) header timestamp cannot occur *after* next
                        // consensus state's height
                        let next_cs = downcast_tm_consensus_state(next_cs.as_ref())?;

                        if header.signed_header.header().time >= next_cs.timestamp {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            }
        }
    }
}
