use core::time::Duration;

use ibc_client_tendermint::client_state::ClientState;
use ibc_core_client_context::client_state::ClientStateCommon;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host_types::identifiers::ChainId;
use ibc_primitives::prelude::*;
use ibc_proto::google::protobuf::Any;
use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;

/// Provides an implementation of `ValidationContext::validate_self_client` for
/// Tendermint-based hosts.
pub trait ValidateSelfClientContext {
    fn validate_self_tendermint_client(
        &self,
        client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ContextError> {
        let tm_client_state = ClientState::try_from(client_state_of_host_on_counterparty)
            .map_err(|_| ConnectionError::InvalidClientState {
                reason: "client must be a tendermint client".to_string(),
            })
            .map_err(ContextError::ConnectionError)?;

        let tm_client_state_inner = tm_client_state.inner();

        tm_client_state_inner
            .validate()
            .map_err(ClientError::from)?;

        if tm_client_state_inner.is_frozen() {
            return Err(ClientError::ClientFrozen {
                description: String::new(),
            }
            .into());
        }

        let self_chain_id = self.chain_id();
        if self_chain_id != &tm_client_state_inner.chain_id {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid chain-id. expected: {}, got: {}",
                        self_chain_id, tm_client_state_inner.chain_id
                    ),
                },
            ));
        }

        let latest_height = tm_client_state.latest_height();
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

        if self.proof_specs() != &tm_client_state_inner.proof_specs {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client has invalid proof specs. expected: {:?}, got: {:?}",
                        self.proof_specs(),
                        tm_client_state_inner.proof_specs
                    ),
                },
            ));
        }

        let _ = {
            let trust_level = tm_client_state_inner.trust_level;

            TendermintTrustThresholdFraction::new(
                trust_level.numerator(),
                trust_level.denominator(),
            )
            .map_err(|_| ConnectionError::InvalidClientState {
                reason: "invalid trust level".to_string(),
            })?
        };

        if self.unbonding_period() != tm_client_state_inner.unbonding_period {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid unbonding period. expected: {:?}, got: {:?}",
                        self.unbonding_period(),
                        tm_client_state_inner.unbonding_period,
                    ),
                },
            ));
        }

        if tm_client_state_inner.unbonding_period < tm_client_state_inner.trusting_period {
            return Err(ContextError::ConnectionError(ConnectionError::InvalidClientState{ reason: format!(
                "unbonding period must be greater than trusting period. unbonding period ({:?}) < trusting period ({:?})",
                tm_client_state_inner.unbonding_period,
                tm_client_state_inner.trusting_period
            )}));
        }

        if !tm_client_state_inner.upgrade_path.is_empty()
            && self.upgrade_path() != tm_client_state_inner.upgrade_path
        {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "invalid upgrade path. expected: {:?}, got: {:?}",
                        self.upgrade_path(),
                        tm_client_state_inner.upgrade_path
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
