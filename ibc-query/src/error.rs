use alloc::string::{String, ToString};

use displaydoc::Display;
use ibc::core::client::types::error::ClientError;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::error::IdentifierError;
use tonic::Status;

#[derive(Debug, Display)]
pub enum QueryError {
    /// Context error: {0}
    ContextError(ContextError),
    /// Client error: {0}
    ClientError(ClientError),
    /// Identifier error: {0}
    IdentifierError(IdentifierError),
    /// Proof not found: {description}
    ProofNotFound { description: String },
}

impl From<QueryError> for Status {
    fn from(e: QueryError) -> Self {
        match e {
            QueryError::ContextError(e) => Status::internal(e.to_string()),
            QueryError::ClientError(e) => Status::internal(e.to_string()),
            QueryError::IdentifierError(e) => Status::internal(e.to_string()),
            QueryError::ProofNotFound { description } => Status::not_found(description),
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
        QueryError::ClientError(e)
    }
}

impl From<IdentifierError> for QueryError {
    fn from(e: IdentifierError) -> Self {
        QueryError::IdentifierError(e)
    }
}
