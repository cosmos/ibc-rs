//! Defines the Non-Fungible Token Transfer (ICS-721) error types.
use core::convert::Infallible;

use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::DecodingError;

use displaydoc::Display;

#[derive(Display, Debug)]
pub enum NftTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// decoding error: `{0}`
    DecodingError(DecodingError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid URI: `{0}`
    InvalidUri(http::uri::InvalidUri),
    /// invalid trace `{0}`
    InvalidTrace(String),
    /// missing destination channel `{channel_id}` on port `{port_id}`
    MissingDestinationChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// empty base class ID
    EmptyBaseClassId,
    /// empty token ID
    EmptyTokenId,
    /// mismatched number of token IDs: expected `{expected}`, actual `{actual}`
    MismatchedNumberOfTokenIds { expected: usize, actual: usize },
    /// mismatched channel orders: expected `{expected}`, actual `{actual}`
    MismatchedChannelOrders { expected: Order, actual: Order },
    /// mismatched port IDs: expected `{expected}`, actual `{actual}`
    MismatchedPortIds { expected: PortId, actual: PortId },
    /// failed to deserialize packet data
    FailedToDeserializePacketData,
    /// failed to deserialize acknowledgement
    FailedToDeserializeAck,
    /// failed to parse account ID
    FailedToParseAccount,
    /// failed to decode raw msg: `{description}`
    FailedToDecodeRawMsg { description: String },
    /// channel cannot be closed
    UnsupportedClosedChannel,
    /// unknown msg type: `{0}`
    UnknownMsgType(String),
}

#[cfg(feature = "std")]
impl std::error::Error for NftTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::InvalidUri(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::DecodingError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Infallible> for NftTransferError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl From<ContextError> for NftTransferError {
    fn from(err: ContextError) -> Self {
        Self::ContextError(err)
    }
}

impl From<IdentifierError> for NftTransferError {
    fn from(err: IdentifierError) -> Self {
        Self::InvalidIdentifier(err)
    }
}

impl From<DecodingError> for NftTransferError {
    fn from(err: DecodingError) -> Self {
        Self::DecodingError(err)
    }
}

impl From<NftTransferError> for StatusValue {
    fn from(err: NftTransferError) -> Self {
        StatusValue::new(err.to_string()).expect("error message must not be empty")
    }
}
