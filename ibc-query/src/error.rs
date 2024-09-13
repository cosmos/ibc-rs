use alloc::string::{String, ToString};

use displaydoc::Display;
use ibc::core::channel::types::error::{ChannelError, PacketError};
use ibc::core::client::types::error::ClientError;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::error::{DecodingError, IdentifierError};
use tonic::Status;

/// The main error type of the ibc-query crate. This type mainly
/// serves to surface lower-level errors that occur when executing
/// ibc-query's codepaths.
#[derive(Debug, Display)]
pub enum QueryError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// decoding error: `{0}`
    DecodingError(DecodingError),
    /// missing proof: `{0}`
    MissingProof(String),
    /// missing field: `{0}`
    MissingField(String),
}

impl QueryError {
    pub fn missing_proof<T: ToString>(description: T) -> Self {
        Self::MissingProof(description.to_string())
    }

    pub fn missing_field<T: ToString>(description: T) -> Self {
        Self::MissingField(description.to_string())
    }
}

impl From<QueryError> for Status {
    fn from(e: QueryError) -> Self {
        match e {
            QueryError::DecodingError(de) => Self::internal(de.to_string()),
            QueryError::ContextError(ctx_err) => Self::internal(ctx_err.to_string()),
            QueryError::MissingProof(description) => Self::not_found(description),
            QueryError::MissingField(description) => Self::invalid_argument(description),
        }
    }
}

impl From<ContextError> for QueryError {
    fn from(e: ContextError) -> Self {
        Self::ContextError(e)
    }
}

impl From<ClientError> for QueryError {
    fn from(e: ClientError) -> Self {
        Self::ContextError(ContextError::ClientError(e))
    }
}

impl From<ConnectionError> for QueryError {
    fn from(e: ConnectionError) -> Self {
        Self::ContextError(ContextError::ConnectionError(e))
    }
}

impl From<ChannelError> for QueryError {
    fn from(e: ChannelError) -> Self {
        Self::ContextError(ContextError::ChannelError(e))
    }
}

impl From<PacketError> for QueryError {
    fn from(e: PacketError) -> Self {
        Self::ContextError(ContextError::PacketError(e))
    }
}

impl From<DecodingError> for QueryError {
    fn from(e: DecodingError) -> Self {
        Self::DecodingError(e)
    }
}

impl From<IdentifierError> for QueryError {
    fn from(e: IdentifierError) -> Self {
        Self::DecodingError(DecodingError::Identifier(e))
    }
}
