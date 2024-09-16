//! Defines the Non-Fungible Token Transfer (ICS-721) error types.
use derive_more::From;
use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::HandlerError;
use ibc_core::host::types::error::{DecodingError, HostError};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;

#[derive(Display, Debug, From)]
pub enum NftTransferError {
    /// handler error: `{0}`
    Handler(HandlerError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// missing destination channel `{channel_id}` on port `{port_id}`
    MissingDestinationChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
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
    /// channel cannot be closed
    UnsupportedClosedChannel,
}

#[cfg(feature = "std")]
impl std::error::Error for NftTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Handler(e) => Some(e),
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<HostError> for NftTransferError {
    fn from(e: HostError) -> Self {
        Self::Handler(HandlerError::Host(e))
    }
}

impl From<NftTransferError> for StatusValue {
    fn from(err: NftTransferError) -> Self {
        StatusValue::new(err.to_string()).expect("error message must not be empty")
    }
}
