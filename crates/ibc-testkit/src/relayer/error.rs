use displaydoc::Display;
use ibc::core::client::types::Height;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::ClientId;

#[derive(Debug, Display)]
pub enum RelayerError {
    /// client state on destination chain not found, (client id: `{client_id}`)
    ClientStateNotFound { client_id: ClientId },
    /// the client on destination chain is already up-to-date (client id: `{client_id}`, source height: `{source_height}`, dest height: `{destination_height}`)
    ClientAlreadyUpToDate {
        client_id: ClientId,
        source_height: Height,
        destination_height: Height,
    },
    /// the client on destination chain is at a higher height (client id: `{client_id}`, source height: `{source_height}`, dest height: `{destination_height}`)
    ClientAtHigherHeight {
        client_id: ClientId,
        source_height: Height,
        destination_height: Height,
    },
    /// transaction processing by modules failed error: `{0}`
    TransactionFailed(ContextError),
    /// connection error: `{0}`
    Connection(ConnectionError),
}

#[cfg(feature = "std")]
impl std::error::Error for RelayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::TransactionFailed(e) => Some(e),
            Self::Connection(e) => Some(e),
            _ => None,
        }
    }
}
