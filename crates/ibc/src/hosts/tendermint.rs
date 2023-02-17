use core::time::Duration;

use alloc::format;
use alloc::string::{String, ToString};
use ibc_proto::google::protobuf::Any;

use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics23_commitment::specs::ProofSpecs;
use crate::core::ics24_host::identifier::ChainId;
use crate::Height;

use tendermint::trust_threshold::TrustThresholdFraction as TendermintTrustThresholdFraction;

/// Provides an implementation of `ValidationContext::validate_self_client` for
/// Tendermint-based hosts.
pub trait ValidateSelfClientContext {
    fn validate_self_tendermint_client(
        &self,
        client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ConnectionError> {
        let tm_client_state = TmClientState::try_from(client_state_of_host_on_counterparty)
            .map_err(|_| ConnectionError::InvalidClientState {
                reason: "client must be a tendermint client".to_string(),
            })?;

        if tm_client_state.is_frozen() {
            return Err(ConnectionError::InvalidClientState {
                reason: "client is frozen".to_string(),
            });
        }

        let self_chain_id = self.chain_id();
        if self_chain_id != &tm_client_state.chain_id {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "invalid chain-id. expected: {}, got: {}",
                    self_chain_id, tm_client_state.chain_id
                ),
            });
        }

        let self_revision_number = self_chain_id.version();
        if self_revision_number != tm_client_state.latest_height().revision_number() {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "client is not in the same revision as the chain. expected: {}, got: {}",
                    self_revision_number,
                    tm_client_state.latest_height().revision_number()
                ),
            });
        }

        if tm_client_state.latest_height() >= self.host_current_height() {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "client has latest height {} greater than or equal to chain height {}",
                    tm_client_state.latest_height(),
                    self.host_current_height()
                ),
            });
        }

        if self.proof_specs() != &tm_client_state.proof_specs {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "client has invalid proof specs. expected: {:?}, got: {:?}",
                    self.proof_specs(),
                    tm_client_state.proof_specs
                ),
            });
        }

        let _ = {
            let trust_level = tm_client_state.trust_level;

            TendermintTrustThresholdFraction::new(
                trust_level.numerator(),
                trust_level.denominator(),
            )
            .map_err(|_| ConnectionError::InvalidClientState {
                reason: "invalid trust level".to_string(),
            })?
        };

        if self.unbonding_period() != tm_client_state.unbonding_period {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "invalid unbonding period. expected: {:?}, got: {:?}",
                    self.unbonding_period(),
                    tm_client_state.unbonding_period,
                ),
            });
        }

        if tm_client_state.unbonding_period < tm_client_state.trusting_period {
            return Err(ConnectionError::InvalidClientState{ reason: format!(
                "unbonding period must be greater than trusting period. unbonding period ({:?}) < trusting period ({:?})",
                tm_client_state.unbonding_period,
                tm_client_state.trusting_period
            )});
        }

        if !tm_client_state.upgrade_path.is_empty()
            && self.upgrade_path() != tm_client_state.upgrade_path
        {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "invalid upgrade path. expected: {:?}, got: {:?}",
                    self.upgrade_path(),
                    tm_client_state.upgrade_path
                ),
            });
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
