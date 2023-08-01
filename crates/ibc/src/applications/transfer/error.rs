//! Defines the token transfer error type

use core::convert::Infallible;
use core::str::Utf8Error;
use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;
use uint::FromDecStrErr;

use crate::core::ics04_channel::acknowledgement::StatusValue;
use crate::core::ics04_channel::channel::Order;
use crate::core::ics24_host::identifier::{ChannelId, IdentifierError, PortId};
use crate::core::ContextError;
use crate::prelude::*;

#[derive(Display, Debug)]
pub enum TokenTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// destination channel not found in the counterparty of port_id `{port_id}` and channel_id `{channel_id}`
    DestinationChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// base denomination is empty
    EmptyBaseDenom,
    /// invalid prot id n trace at position: `{pos}`, validation error: `{validation_error}`
    InvalidTracePortId {
        pos: u64,
        validation_error: IdentifierError,
    },
    /// invalid channel id in trace at position: `{pos}`, validation error: `{validation_error}`
    InvalidTraceChannelId {
        pos: u64,
        validation_error: IdentifierError,
    },
    /// trace length must be even but got: `{len}`
    InvalidTraceLength { len: u64 },
    /// invalid amount error: `{0}`
    InvalidAmount(FromDecStrErr),
    /// invalid token
    InvalidToken,
    /// expected `{expect_order}` channel, got `{got_order}`
    ChannelNotUnordered {
        expect_order: Order,
        got_order: Order,
    },
    /// channel cannot be closed
    CantCloseChannel,
    /// failed to deserialize packet data
    PacketDataDeserialization,
    /// failed to deserialize acknowledgement
    AckDeserialization,
    /// receive is not enabled
    ReceiveDisabled { reason: String },
    /// send is not enabled
    SendDisabled { reason: String },
    /// failed to parse as AccountId
    ParseAccountFailure,
    /// invalid port: `{port_id}`, expected `{exp_port_id}`
    InvalidPort {
        port_id: PortId,
        exp_port_id: PortId,
    },
    /// decoding raw msg error: `{0}`
    DecodeRawMsg(TendermintProtoError),
    /// unknown msg type: `{msg_type}`
    UnknownMsgType { msg_type: String },
    /// invalid coin string: `{coin}`
    InvalidCoin { coin: String },
    /// decoding raw bytes as UTF8 string error: `{0}`
    Utf8Decode(Utf8Error),
}

#[cfg(feature = "std")]
impl std::error::Error for TokenTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::InvalidTracePortId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidTraceChannelId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidAmount(e) => Some(e),
            Self::DecodeRawMsg(e) => Some(e),
            Self::Utf8Decode(e) => Some(e),
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
