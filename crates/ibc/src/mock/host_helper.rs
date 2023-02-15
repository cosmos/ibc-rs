use alloc::format;
use alloc::string::ToString;
use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics24_host::identifier::ChainId;
use crate::mock::client_state::MockClientState;
use crate::Height;

/// Provides an implementation of `ValidationContext::validate_self_client` for
/// the mock host.
pub trait ValidateSelfMockClientContext {
    fn validate_self_mock_client(
        &self,
        host_client_state_on_counterparty: Any,
    ) -> Result<(), ConnectionError> {
        let host_client_state_on_counterparty =
            MockClientState::try_from(host_client_state_on_counterparty).map_err(|_| {
                ConnectionError::InvalidClientState {
                    reason: "client must be a mock client".to_string(),
                }
            })?;

        if host_client_state_on_counterparty.is_frozen() {
            return Err(ConnectionError::InvalidClientState {
                reason: "client is frozen".to_string(),
            });
        }

        let self_chain_id = self.chain_id();
        let self_revision_number = self_chain_id.version();
        if self_revision_number
            != host_client_state_on_counterparty
                .latest_height()
                .revision_number()
        {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "client is not in the same revision as the chain. expected: {}, got: {}",
                    self_revision_number,
                    host_client_state_on_counterparty
                        .latest_height()
                        .revision_number()
                ),
            });
        }

        if host_client_state_on_counterparty.latest_height() >= self.host_height() {
            return Err(ConnectionError::InvalidClientState {
                reason: format!(
                    "client has latest height {} greater than or equal to chain height {}",
                    host_client_state_on_counterparty.latest_height(),
                    self.host_height()
                ),
            });
        }

        Ok(())
    }

    /// Returns the host chain id
    fn chain_id(&self) -> &ChainId;

    /// Returns the host current height
    fn host_height(&self) -> Height;
}
