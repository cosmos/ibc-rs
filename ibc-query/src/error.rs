use alloc::string::{String, ToString};

use displaydoc::Display;
use ibc::core::channel::types::error::ChannelError;
use ibc::core::client::types::error::ClientError;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::handler::types::error::HandlerError;
use ibc::core::host::types::error::{DecodingError, HostError, IdentifierError};
use tonic::Status;

/// The main error type of the ibc-query crate. This type mainly
/// serves to surface lower-level errors that occur when executing
/// ibc-query's codepaths.
#[derive(Debug, Display)]
pub enum QueryError {
    /// handler error: `{0}`
    Handler(HandlerError),
    /// host error: `{0}`
    Host(HostError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
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
            QueryError::Handler(ctx_err) => Self::internal(ctx_err.to_string()),
            QueryError::Host(host_err) => Self::internal(host_err.to_string()),
            QueryError::Decoding(de) => Self::internal(de.to_string()),
            QueryError::MissingProof(description) => Self::not_found(description),
            QueryError::MissingField(description) => Self::invalid_argument(description),
        }
    }
}

impl From<ClientError> for QueryError {
    fn from(e: ClientError) -> Self {
        Self::Handler(HandlerError::Client(e))
    }
}

impl From<ConnectionError> for QueryError {
    fn from(e: ConnectionError) -> Self {
        Self::Handler(HandlerError::Connection(e))
    }
}

impl From<ChannelError> for QueryError {
    fn from(e: ChannelError) -> Self {
        Self::Handler(HandlerError::Channel(e))
    }
}

impl From<DecodingError> for QueryError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<IdentifierError> for QueryError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

impl From<HostError> for QueryError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}
