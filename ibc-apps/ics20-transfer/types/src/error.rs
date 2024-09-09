//! Defines the token transfer error type
use core::convert::Infallible;

use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::HandlerError;
use ibc_core::host::types::error::{DecodingError, HostError, IdentifierError};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use uint::FromDecStrErr;

#[derive(Display, Debug)]
pub enum TokenTransferError {
    /// handler error: `{0}`
    Handler(HandlerError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// identifier error: `{0}`
    Identifier(IdentifierError),
    /// invalid amount: `{0}`
    InvalidAmount(FromDecStrErr),
    /// invalid coin: `{0}`
    InvalidCoin(String),
    /// invalid trace: `{0}`
    InvalidTrace(String),
    /// missing token
    MissingToken,
    /// missing destination channel `{channel_id}` on port `{port_id}`
    MissingDestinationChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// mismatched channel orders: expected `{expected}`, actual `{actual}`
    MismatchedChannelOrders { expected: Order, actual: Order },
    /// mismatched port IDs: expected `{expected}`, actual `{actual}`
    MismatchedPortIds { expected: PortId, actual: PortId },
    /// channel cannot be closed
    UnsupportedClosedChannel,
    /// empty base denomination
    EmptyBaseDenom,

    // TODO(seanchen1991): Used in basecoin; this variant should be moved
    // to a host-relevant error
    /// failed to parse account ID
    FailedToParseAccount,
}

#[cfg(feature = "std")]
impl std::error::Error for TokenTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Handler(e) => Some(e),
            Self::Identifier(e) => Some(e),
            Self::InvalidAmount(e) => Some(e),
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Infallible> for TokenTransferError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl From<HandlerError> for TokenTransferError {
    fn from(e: HandlerError) -> Self {
        Self::Handler(e)
    }
}

impl From<IdentifierError> for TokenTransferError {
    fn from(e: IdentifierError) -> Self {
        Self::Identifier(e)
    }
}

impl From<DecodingError> for TokenTransferError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<HostError> for TokenTransferError {
    fn from(e: HostError) -> Self {
        Self::Handler(HandlerError::Host(e))
    }
}

impl From<TokenTransferError> for StatusValue {
    fn from(e: TokenTransferError) -> Self {
        StatusValue::new(e.to_string()).expect("error message must not be empty")
    }
}
