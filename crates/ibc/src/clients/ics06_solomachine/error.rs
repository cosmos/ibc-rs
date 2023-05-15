use crate::core::ics04_channel::error::ChannelError;
use crate::prelude::*;

use crate::core::ics02_client::error::ClientError;
use crate::core::timestamp::ParseTimestampError;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// dummy error
    Dummy,
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// consensus state PublicKey is None
    EmptyConsensusStatePublicKey,
    /// invlid height
    InvalidHeight(ClientError),
    /// invalid raw client id: `{client_id}`
    InvalidRawClientId { client_id: String },
    /// unknow data type: `{0}`
    UnknownDataType(i32),
    /// prase time error
    ParseTimeError(ParseTimestampError),
    /// Channel error: `{0}`
    ChannelError(ChannelError),
    /// timestamp cannot be 0
    TimeStampIsEmpty,
    /// diversifier cannot contain only spaces
    DriversifierContainOnlySpaces,
    /// public key parsed failed: `{0}`
    PublicKeyParseFailed(eyre::ErrReport),
    /// public key is empty
    PublicKeyIsEmpty,
    /// consensus state is empty
    ConsensusStateIsEmpty,
    /// SignatureAndData empty
    SignatureAndDataIsEmpty,
    /// Sequence cannot be zero
    SequenceCannotZero,
    /// Proof cannot be empty
    ProofCannotEmpty,
    /// Other : `{0}`
    Other(String),
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
