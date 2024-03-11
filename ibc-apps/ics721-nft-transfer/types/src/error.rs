//! Defines the Non-Fungible Token Transfer (ICS-721) error types.
use core::convert::Infallible;
use core::str::Utf8Error;

use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;

#[derive(Display, Debug)]
pub enum NftTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid URI: `{uri}`, validation error: `{validation_error}``
    InvalidUri {
        uri: String,
        validation_error: http::uri::InvalidUri,
    },
    /// destination channel not found in the counterparty of port_id `{port_id}` and channel_id `{channel_id}`
    DestinationChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// base class ID is empty
    EmptyBaseClassId,
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
    /// no token ID
    NoTokenId,
    /// invalid token ID
    InvalidTokenId,
    /// duplicated token IDs
    DuplicatedTokenIds,
    /// The length of token IDs mismatched that of token URIs or token data
    TokenMismatched,
    /// invalid json data
    InvalidJsonData,
    /// the data is not in the JSON format specified by ICS-721
    InvalidIcs721Data,
    /// expected `{expect_order}` channel, got `{got_order}`
    ChannelNotUnordered {
        expect_order: Order,
        got_order: Order,
    },
    /// channel cannot be closed
    CantCloseChannel,
    /// `{sender}` doesn't own the NFT
    InvalidOwner { sender: String },
    /// owner is not found
    OwnerNotFound,
    /// nft is not found
    NftNotFound,
    /// nft class is not found
    NftClassNotFound,
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
    /// decoding raw msg error: `{reason}`
    DecodeRawMsg { reason: String },
    /// unknown msg type: `{msg_type}`
    UnknownMsgType { msg_type: String },
    /// decoding raw bytes as UTF8 string error: `{0}`
    Utf8Decode(Utf8Error),
    /// other error: `{0}`
    Other(String),
}

#[cfg(feature = "std")]
impl std::error::Error for NftTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::InvalidUri {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidIdentifier(e)
            | Self::InvalidTracePortId {
                validation_error: e,
                ..
            }
            | Self::InvalidTraceChannelId {
                validation_error: e,
                ..
            } => Some(e),
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
    fn from(err: ContextError) -> NftTransferError {
        Self::ContextError(err)
    }
}

impl From<IdentifierError> for NftTransferError {
    fn from(err: IdentifierError) -> NftTransferError {
        Self::InvalidIdentifier(err)
    }
}

impl From<NftTransferError> for StatusValue {
    fn from(err: NftTransferError) -> Self {
        StatusValue::new(err.to_string()).expect("error message must not be empty")
    }
}
