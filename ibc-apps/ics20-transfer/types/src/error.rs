//! Defines the token transfer error type
use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::DecodingError;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;

#[derive(Display, Debug)]
pub enum TokenTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
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
    /// failed to deserialize packet data
    FailedToDeserializePacketData,
    /// failed to deserialize acknowledgement
    FailedToDeserializeAck,

    // TODO(seanchen1991): Used in basecoin; this variant should be moved
    // to a host-relevant error
    /// failed to parse account ID
    FailedToParseAccount,
}

#[cfg(feature = "std")]
impl std::error::Error for TokenTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ContextError> for TokenTransferError {
    fn from(e: ContextError) -> Self {
        Self::ContextError(e)
    }
}

impl From<DecodingError> for TokenTransferError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<TokenTransferError> for StatusValue {
    fn from(e: TokenTransferError) -> Self {
        StatusValue::new(e.to_string()).expect("error message must not be empty")
    }
}
