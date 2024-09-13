use core::time::Duration;

use ibc_client_tendermint::types::ClientState as TmClientState;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_host_types::error::HostError;
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
    ) -> Result<(), HostError> {
        client_state_of_host_on_counterparty
            .validate()
            .map_err(|e| HostError::FailedToValidateClient {
                description: e.to_string(),
            })?;

        if client_state_of_host_on_counterparty.is_frozen() {
            return Err(HostError::invalid_state(
                "client unexpectedly frozen".to_string(),
            ));
        }

        let self_chain_id = self.chain_id();

        if self_chain_id != &client_state_of_host_on_counterparty.chain_id {
            return Err(HostError::invalid_state(format!(
                "invalid chain ID: expected {}, actual {}",
                self_chain_id, client_state_of_host_on_counterparty.chain_id
            )));
        }

        let latest_height = client_state_of_host_on_counterparty.latest_height;
        let self_revision_number = self_chain_id.revision_number();

        if self_revision_number != latest_height.revision_number() {
            return Err(HostError::invalid_state(format!(
                "mismatched client revision numbers; expected {}, actual {}",
                self_revision_number,
                latest_height.revision_number()
            )));
        }

        if latest_height >= self.host_current_height() {
            return Err(HostError::invalid_state(format!(
                "client latest height {} should be less than chain height {}",
                latest_height,
                self.host_current_height()
            )));
        }

        if self.proof_specs() != &client_state_of_host_on_counterparty.proof_specs {
            return Err(HostError::invalid_state(format!(
                "invalid client proof specs; expected {:?}, actual {:?}",
                self.proof_specs(),
                client_state_of_host_on_counterparty.proof_specs
            )));
        }

        let _ = {
            let trust_level = client_state_of_host_on_counterparty.trust_level;

            TendermintTrustThresholdFraction::new(
                trust_level.numerator(),
                trust_level.denominator(),
            )
            .map_err(|e| HostError::invalid_state(e.to_string()))?
        };

        if self.unbonding_period() != client_state_of_host_on_counterparty.unbonding_period {
            return Err(HostError::invalid_state(format!(
                "invalid unbonding period; expected {:?}, actual {:?}",
                self.unbonding_period(),
                client_state_of_host_on_counterparty.unbonding_period,
            )));
        }

        if client_state_of_host_on_counterparty.unbonding_period
            < client_state_of_host_on_counterparty.trusting_period
        {
            return Err(HostError::invalid_state(format!(
                "invalid counterparty client state: unbonding period must be greater than trusting period; unbonding period ({:?}) < trusting period ({:?})",
                client_state_of_host_on_counterparty.unbonding_period,
                client_state_of_host_on_counterparty.trusting_period
            )));
        }

        if !client_state_of_host_on_counterparty.upgrade_path.is_empty()
            && self.upgrade_path() != client_state_of_host_on_counterparty.upgrade_path
        {
            return Err(HostError::invalid_state(format!(
                "invalid upgrade path; expected {:?}, actual {:?}",
                self.upgrade_path(),
                client_state_of_host_on_counterparty.upgrade_path
            )));
        }

        Ok(())
    }

    /// Returns the chain id of the host
    fn chain_id(&self) -> &ChainId;

    /// Returns the host current height
    fn host_current_height(&self) -> Height;

    /// Returns the proof specs of the host
    fn proof_specs(&self) -> &ProofSpecs;

    /// Returns the unbonding period of the host
    fn unbonding_period(&self) -> Duration;

    /// Returns the host upgrade path. May be empty.
    fn upgrade_path(&self) -> &[String];
}
