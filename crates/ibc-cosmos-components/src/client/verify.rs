use alloc::format;

use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::ics07_tendermint::error::{Error, IntoResult};
use ibc::clients::ics07_tendermint::header::Header as TmHeader;
use ibc::clients::ics07_tendermint::ValidationContext as TmValidationContext;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics24_host::identifier::{ChainId, ClientId};
use ibc::core::ics24_host::path::ClientConsensusStatePath;
use ibc::prelude::ToString;
use tendermint_light_client_verifier::options::Options;
use tendermint_light_client_verifier::types::{TrustedBlockState, UntrustedBlockState};
use tendermint_light_client_verifier::{ProdVerifier, Verifier};

pub trait HasClientStateMethods {
    fn chain_id(&self) -> &ChainId;

    fn light_client_options(&self) -> Options;

    fn verifier(&self) -> &ProdVerifier;
}

pub fn verify_header<ClientValidationContext, ClientState>(
    client_state: &ClientState,
    ctx: &ClientValidationContext,
    client_id: &ClientId,
    header: TmHeader,
) -> Result<(), ClientError>
where
    ClientState: HasClientStateMethods,
    ClientValidationContext: TmValidationContext,
{
    // Checks that the header fields are valid.
    header.validate_basic()?;

    // The tendermint-light-client crate though works on heights that are assumed
    // to have the same revision number. We ensure this here.
    header.verify_chain_id_version_matches_height(client_state.chain_id())?;

    // Delegate to tendermint-light-client, which contains the required checks
    // of the new header against the trusted consensus state.
    {
        let trusted_state = {
            let trusted_client_cons_state_path =
                ClientConsensusStatePath::new(client_id, &header.trusted_height);
            let trusted_consensus_state: TmConsensusState = ctx
                .consensus_state(&trusted_client_cons_state_path)?
                .try_into()
                .map_err(|err| ClientError::Other {
                    description: err.to_string(),
                })?;

            check_header_trusted_next_validator_set(&header, &trusted_consensus_state)?;

            TrustedBlockState {
                chain_id: &client_state
                    .chain_id()
                    .to_string()
                    .try_into()
                    .map_err(|e| ClientError::Other {
                        description: format!("failed to parse chain id: {}", e),
                    })?,
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

        let options = client_state.light_client_options();
        let now =
            ctx.host_timestamp()?
                .into_tm_time()
                .ok_or_else(|| ClientError::ClientSpecific {
                    description: "host timestamp is not a valid TM timestamp".to_string(),
                })?;

        // main header verification, delegated to the tendermint-light-client crate.
        client_state
            .verifier()
            .verify_update_header(untrusted_state, trusted_state, &options, now)
            .into_result()?;
    }

    Ok(())
}

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
