use alloc::string::{String, ToString};

use displaydoc::Display;
use ibc::core::channel::types::error::{ChannelError, PacketError};
use ibc::core::client::types::error::ClientError;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::error::IdentifierError;
use tonic::Status;

#[derive(Debug, Display)]
pub enum QueryError {
    /// Context error: {0}
    ContextError(ContextError),
    /// Identifier error: {0}
    IdentifierError(IdentifierError),
    /// Proof not found: {0}
    ProofNotFound(String),
    /// Missing field: {0}
    MissingField(String),
}

impl QueryError {
    pub fn proof_not_found(description: impl ToString) -> Self {
        QueryError::ProofNotFound(description.to_string())
    }

    pub fn missing_field(description: impl ToString) -> Self {
        QueryError::MissingField(description.to_string())
    }
}

impl From<QueryError> for Status {
    fn from(e: QueryError) -> Self {
        match e {
            QueryError::ContextError(e) => Status::internal(e.to_string()),
            QueryError::IdentifierError(e) => Status::internal(e.to_string()),
            QueryError::ProofNotFound(description) => Status::not_found(description),
            QueryError::MissingField(description) => Status::invalid_argument(description),
        }
    }
}

impl From<ContextError> for QueryError {
    fn from(e: ContextError) -> Self {
        QueryError::ContextError(e)
    }
}

impl From<ClientError> for QueryError {
    fn from(e: ClientError) -> Self {
        QueryError::ContextError(ContextError::ClientError(e))
    }
}

impl From<ConnectionError> for QueryError {
    fn from(e: ConnectionError) -> Self {
        QueryError::ContextError(ContextError::ConnectionError(e))
    }
}

impl From<ChannelError> for QueryError {
    fn from(e: ChannelError) -> Self {
        QueryError::ContextError(ContextError::ChannelError(e))
    }
}

impl From<PacketError> for QueryError {
    fn from(e: PacketError) -> Self {
        QueryError::ContextError(ContextError::PacketError(e))
    }
}

impl From<IdentifierError> for QueryError {
    fn from(e: IdentifierError) -> Self {
        QueryError::IdentifierError(e)
    }
}
