use alloc::string::{String, ToString};

use displaydoc::Display;
use ibc::core::channel::types::error::{ChannelError, PacketError};
use ibc::core::client::types::error::ClientError;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::HandlerError;
use ibc::core::host::types::error::IdentifierError;
use tonic::Status;

/// The main error type of the ibc-query crate. This type mainly
/// serves to surface lower-level errors that occur when executing
/// ibc-query's codepaths.
#[derive(Debug, Display)]
pub enum QueryError {
    /// context error: `{0}`
    HandlerError(HandlerError),
    /// identifier error: `{0}`
    IdentifierError(IdentifierError),
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
            QueryError::HandlerError(ctx_err) => Self::internal(ctx_err.to_string()),
            QueryError::IdentifierError(id_err) => Self::internal(id_err.to_string()),
            QueryError::MissingProof(description) => Self::not_found(description),
            QueryError::MissingField(description) => Self::invalid_argument(description),
        }
    }
}

impl From<HandlerError> for QueryError {
    fn from(e: HandlerError) -> Self {
        Self::HandlerError(e)
    }
}

impl From<ClientError> for QueryError {
    fn from(e: ClientError) -> Self {
        Self::HandlerError(HandlerError::ClientError(e))
    }
}

impl From<ConnectionError> for QueryError {
    fn from(e: ConnectionError) -> Self {
        Self::HandlerError(HandlerError::ConnectionError(e))
    }
}

impl From<ChannelError> for QueryError {
    fn from(e: ChannelError) -> Self {
        Self::HandlerError(HandlerError::ChannelError(e))
    }
}

impl From<PacketError> for QueryError {
    fn from(e: PacketError) -> Self {
        Self::HandlerError(HandlerError::PacketError(e))
    }
}

impl From<IdentifierError> for QueryError {
    fn from(e: IdentifierError) -> Self {
        Self::IdentifierError(e)
    }
}
