//! Defines the token transfer error type
use core::convert::Infallible;

use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use uint::FromDecStrErr;

#[derive(Display, Debug)]
pub enum TokenTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid trace: `{0}`
    InvalidTrace(String),
    /// invalid amount: `{0}`
    InvalidAmount(FromDecStrErr),
    /// invalid token
    InvalidToken,
    /// invalid coin: `{actual}`
    InvalidCoin { actual: String },
    /// missing destination channel `{channel_id}` on port `{port_id}`
    MissingDestinationChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// mismatched channel orders: expected `{expected}`, actual `{actual}`
    MismatchedChannelOrders { expected: Order, actual: Order },
    /// mismatched port IDs: expected `{expected}`, actual `{actual}`
    MismatchedPortIds { expected: PortId, actual: PortId },
    /// failed to deserialize packet data
    FailedToDeserializePacket,
    /// failed to deserialize acknowledgement
    FailedToDeserializeAck,
    /// failed to parse account ID
    FailedToParseAccount,
    /// failed to decode raw msg: `{description}`
    FailedToDecodeRawMsg { description: String },
    /// channel cannot be closed
    ClosedChannel,
    /// empty base denomination
    EmptyBaseDenom,
    /// unknown msg type: `{0}`
    UnknownMsgType(String),
}

#[cfg(feature = "std")]
impl std::error::Error for TokenTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::InvalidAmount(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Infallible> for TokenTransferError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl From<ContextError> for TokenTransferError {
    fn from(err: ContextError) -> TokenTransferError {
        Self::ContextError(err)
    }
}

impl From<IdentifierError> for TokenTransferError {
    fn from(err: IdentifierError) -> TokenTransferError {
        Self::InvalidIdentifier(err)
    }
}

impl From<TokenTransferError> for StatusValue {
    fn from(err: TokenTransferError) -> Self {
        StatusValue::new(err.to_string()).expect("error message must not be empty")
    }
}
