use crate::prelude::*;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::Height;
use core::time::Duration;

use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// dummy error
    Dummy,
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// consensus state PublicKey is None
    EmptyConsensusStatePublicKey,
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
        Self::ClientSpecific {
            description: e.to_string(),
        }
    }
}

pub(crate) trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}
