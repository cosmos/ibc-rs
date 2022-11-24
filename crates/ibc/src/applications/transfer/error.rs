use alloc::string::FromUtf8Error;

use core::convert::Infallible;
use core::str::Utf8Error;
use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;
use subtle_encoding::Error as EncodingError;
use uint::FromDecStrErr;

use crate::core::ics04_channel::channel::Order;
use crate::core::ics04_channel::error as channel_error;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::prelude::*;
use crate::signer::SignerError;

#[derive(Display, Debug)]
pub enum TokenTransferError {
    /// unrecognized ICS-20 transfer message type URL `{url}`
    UnknowMessageTypeUrl { url: String },
    /// ICS04 Packet error
    PacketError(channel_error::PacketError),
    /// destination channel not found in the counterparty of port_id `{port_id}` and channel_id `{channel_id}`
    DestinationChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// invalid port identifier `{context}`
    InvalidPortId {
        context: String,
        validation_error: ValidationError,
    },
    /// invalid channel identifier `{context}`
    InvalidChannelId {
        context: String,
        validation_error: ValidationError,
    },
    /// invalid packet timeout height value `{context}`
    InvalidPacketTimeoutHeight { context: String },
    /// invalid packet timeout timestamp value `{timestamp}`
    InvalidPacketTimeoutTimestamp { timestamp: u64 },
    /// utf8 decoding error
    Utf8(FromUtf8Error),
    /// base denomination is empty
    EmptyBaseDenom,
    /// invalid prot id n trace at postion: `{pos}`
    InvalidTracePortId {
        pos: usize,
        validation_error: ValidationError,
    },
    /// invalid channel id in trace at position: `{pos}`
    InvalidTraceChannelId {
        pos: usize,
        validation_error: ValidationError,
    },
    /// trace length must be even but got: `{len}`
    InvalidTraceLength { len: usize },
    /// invalid amount error
    InvalidAmount(FromDecStrErr),
    /// invalid token
    InvalidToken,
    /// failed to parse signer error
    Signer(SignerError),
    /// missing 'ibc/' prefix in denomination
    MissingDenomIbcPrefix,
    /// hashed denom must be of the form 'ibc/Hash'
    MalformedHashDenom,
    /// invalid hex string error
    ParseHex(EncodingError),
    /// expected `{expect_order}` channel, got `{got_order}`
    ChannelNotUnordered {
        expect_order: Order,
        got_order: Order,
    },
    /// expected version `{expect_version}` , got `{got_version}`
    InvalidVersion {
        expect_version: Version,
        got_version: Version,
    },
    /// expected counterparty version `{expect_version}`, got `{got_version}`
    InvalidCounterpartyVersion {
        expect_version: Version,
        got_version: Version,
    },
    /// channel cannot be closed
    CantCloseChannel,
    /// failed to deserialize packet data
    PacketDataDeserialization,
    /// failed to deserialize acknowledgement
    AckDeserialization,
    /// receive is not enabled
    ReceiveDisabled,
    /// send is not enabled
    SendDisabled,
    /// failed to parse as AccountId
    ParseAccountFailure,
    /// invalid port: `{port_id}`, expected `{exp_port_id}`
    InvalidPort {
        port_id: PortId,
        exp_port_id: PortId,
    },
    /// no trace associated with specified hash
    TraceNotFound,
    /// error decoding raw msg
    DecodeRawMsg(TendermintProtoError),
    /// unknown msg type: `{msg_type}`
    UnknownMsgType { msg_type: String },
    /// invalid coin string: `{coin}`
    InvalidCoin { coin: String },
    /// error decoding raw bytes as UTF8 string
    Utf8Decode(Utf8Error),
}

#[cfg(feature = "std")]
impl std::error::Error for TokenTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::PacketError(e) => Some(e),
            Self::InvalidPortId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidChannelId {
                validation_error: e,
                ..
            } => Some(e),
            Self::Utf8(e) => Some(e),
            Self::InvalidTracePortId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidTraceChannelId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidAmount(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::ParseHex(e) => Some(e),
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
