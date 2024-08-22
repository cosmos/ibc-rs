use displaydoc::Display;

use ibc::core::client::types::Height;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::ClientId;

#[derive(Debug, Display)]
pub enum RelayerError {
    /// unnecessary update attempted for client `{0}`: source and destination heights already match
    UnnecessaryClientUpdate(ClientId),
    /// insufficient update height for client `{client_id}` on the source chain; needs to exceed `{destination_height}`
    InsufficientUpdateHeight {
        client_id: ClientId,
        destination_height: Height,
    },
    /// failed to process transaction: `{0}`
    FailedToProcessTransaction(ContextError),
}

#[cfg(feature = "std")]
impl std::error::Error for RelayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::FailedToProcessTransaction(e) => Some(e),
            _ => None,
        }
    }
}
