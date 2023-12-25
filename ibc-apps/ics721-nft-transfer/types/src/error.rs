//! Defines the Non-Fungible Token Transfer (ICS-721) error types.
use core::convert::Infallible;

use displaydoc::Display;
use ibc_core::channel::types::acknowledgement::StatusValue;
use ibc_core::channel::types::channel::Order;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::error::IdentifierError;
use ibc_core::host::types::identifiers::PortId;
use ibc_core::primitives::prelude::*;

#[derive(Display, Debug)]
pub enum NftTransferError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid URI: `{uri}`, validation error: `{error}`
    InvalidUri {
        uri: String,
        error: http::uri::InvalidUri,
    },
    /// expected `{expect_order}` channel, got `{got_order}`
    ChannelNotUnordered {
        expect_order: Order,
        got_order: Order,
    },
    /// channel cannot be closed
    CantCloseChannel,
    /// invalid port: `{port_id}`, expected `{exp_port_id}`
    InvalidPort {
        port_id: PortId,
        exp_port_id: PortId,
    },
    /// invalid token
    InvalidTokenId,
    /// decoding raw msg error: `{reason}`
    DecodeRawMsg { reason: String },
    /// unknown msg type: `{msg_type}`
    UnknownMsgType { msg_type: String },
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
}

#[cfg(feature = "std")]
impl std::error::Error for NftTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::InvalidUri { uri: _, error } => Some(error),
            Self::InvalidTracePortId {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidTraceChannelId {
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
