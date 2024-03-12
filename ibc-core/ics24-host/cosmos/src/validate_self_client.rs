use core::time::Duration;

use ibc_client_tendermint::types::ClientState as TmClientState;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host_types::identifiers::ChainId;
use ibc_primitives::prelude::*;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;

/// Provides a default implementation intended for implementing the
/// `ValidationContext::validate_self_client` API.
///
/// This validation logic tailored for Tendermint client states of a host chain
/// operating across various counterparty chains.
pub trait ValidateSelfClientContext {
    fn validate_self_tendermint_client(
        &self,
        client_state_of_host_on_counterparty: TmClientState,
    ) -> Result<(), ContextError> {
        client_state_of_host_on_counterparty
            .validate()
            .map_err(ClientError::from)?;

        if client_state_of_host_on_counterparty.is_frozen() {
            return Err(ClientError::ClientFrozen {
                description: String::new(),
            }
            .into());
        }

        let self_chain_id = self.chain_id();
        if self_chain_id != &client_state_of_host_on_counterparty.chain_id {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid chain-id. expected: {}, got: {}",
                        self_chain_id, client_state_of_host_on_counterparty.chain_id
                    ),
                },
            ));
        }

        let latest_height = client_state_of_host_on_counterparty.latest_height;
        let self_revision_number = self_chain_id.revision_number();
        if self_revision_number != latest_height.revision_number() {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client is not in the same revision as the chain. expected: {}, got: {}",
                        self_revision_number,
                        latest_height.revision_number()
                    ),
                },
            ));
        }

        if latest_height >= self.host_current_height() {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client has latest height {} greater than or equal to chain height {}",
                        latest_height,
                        self.host_current_height()
                    ),
                },
            ));
        }

        if self.proof_specs() != &client_state_of_host_on_counterparty.proof_specs {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client has invalid proof specs. expected: {:?}, got: {:?}",
                        self.proof_specs(),
                        client_state_of_host_on_counterparty.proof_specs
                    ),
                },
            ));
        }

        let _ = {
            let trust_level = client_state_of_host_on_counterparty.trust_level;

            TendermintTrustThresholdFraction::new(
                trust_level.numerator(),
                trust_level.denominator(),
            )
            .map_err(|_| ConnectionError::InvalidClientState {
                reason: "invalid trust level".to_string(),
            })?
        };

        if self.unbonding_period() != client_state_of_host_on_counterparty.unbonding_period {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid unbonding period. expected: {:?}, got: {:?}",
                        self.unbonding_period(),
                        client_state_of_host_on_counterparty.unbonding_period,
                    ),
                },
            ));
        }

        if client_state_of_host_on_counterparty.unbonding_period
            < client_state_of_host_on_counterparty.trusting_period
        {
            return Err(ContextError::ConnectionError(ConnectionError::InvalidClientState{ reason: format!(
                "unbonding period must be greater than trusting period. unbonding period ({:?}) < trusting period ({:?})",
                client_state_of_host_on_counterparty.unbonding_period,
                client_state_of_host_on_counterparty.trusting_period
            )}));
        }

        if !client_state_of_host_on_counterparty.upgrade_path.is_empty()
            && self.upgrade_path() != client_state_of_host_on_counterparty.upgrade_path
        {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid upgrade path. expected: {:?}, got: {:?}",
                        self.upgrade_path(),
                        client_state_of_host_on_counterparty.upgrade_path
                    ),
                },
            ));
        }

        Ok(())
    }

    /// Returns the host chain id
    fn chain_id(&self) -> &ChainId;

    /// Returns the host current height
    fn host_current_height(&self) -> Height;

    /// Returns the host proof specs
    fn proof_specs(&self) -> &ProofSpecs;

    /// Returns the host unbonding period
    fn unbonding_period(&self) -> Duration;

    /// Returns the host upgrade path. May be empty.
    fn upgrade_path(&self) -> &[String];
}
