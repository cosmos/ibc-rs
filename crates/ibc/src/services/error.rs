use alloc::string::ToString;

use crate::core::{ics24_host::identifier::IdentifierError, ContextError};
use tonic::Status;

impl From<IdentifierError> for Status {
    fn from(err: IdentifierError) -> Self {
        Status::invalid_argument(err.to_string())
    }
}

impl From<ContextError> for Status {
    fn from(err: ContextError) -> Self {
        Status::not_found(err.to_string())
    }
}
