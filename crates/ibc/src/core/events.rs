//! Events emitted during message handling

use crate::prelude::*;

use core::convert::{TryFrom, TryInto};
use displaydoc::Display;
use tendermint::abci;

use crate::core::ics02_client::error as client_error;
use crate::core::ics02_client::events::{self as ClientEvents};
use crate::core::ics03_connection::error as connection_error;
use crate::core::ics03_connection::events as ConnectionEvents;
use crate::core::ics04_channel::error as channel_error;
use crate::core::ics04_channel::events as ChannelEvents;
use crate::core::ics24_host::error::IdentifierError;
use crate::core::timestamp::ParseTimestampError;

/// All error variants related to IBC events
#[derive(Debug, Display)]
pub enum Error {
    /// error parsing height
    Height,
    /// parse error: `{0}`
    Parse(IdentifierError),
    /// client error: `{0}`
    Client(client_error::ClientError),
    /// connection error: `{0}`
    Connection(connection_error::ConnectionError),
    /// channel error: `{0}`
    Channel(channel_error::ChannelError),
    /// parsing timestamp error: `{0}`
    Timestamp(ParseTimestampError),
    /// decoding protobuf error: `{0}`
    Decode(prost::DecodeError),
    /// incorrect event type: `{event}`
    IncorrectEventType { event: String },
    /// module event cannot use core event types: `{event:?}`
    MalformedModuleEvent { event: ModuleEvent },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Parse(e) => Some(e),
            Self::Client(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::Timestamp(e) => Some(e),
            Self::Decode(e) => Some(e),
            _ => None,
        }
    }
}

const MESSAGE_EVENT: &str = "message";

/// Events created by the IBC component of a chain, destined for a relayer.
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IbcEvent {
    CreateClient(ClientEvents::CreateClient),
    UpdateClient(ClientEvents::UpdateClient),
    UpgradeClient(ClientEvents::UpgradeClient),
    ClientMisbehaviour(ClientEvents::ClientMisbehaviour),

    OpenInitConnection(ConnectionEvents::OpenInit),
    OpenTryConnection(ConnectionEvents::OpenTry),
    OpenAckConnection(ConnectionEvents::OpenAck),
    OpenConfirmConnection(ConnectionEvents::OpenConfirm),

    OpenInitChannel(ChannelEvents::OpenInit),
    OpenTryChannel(ChannelEvents::OpenTry),
    OpenAckChannel(ChannelEvents::OpenAck),
    OpenConfirmChannel(ChannelEvents::OpenConfirm),
    CloseInitChannel(ChannelEvents::CloseInit),
    CloseConfirmChannel(ChannelEvents::CloseConfirm),

    SendPacket(ChannelEvents::SendPacket),
    ReceivePacket(ChannelEvents::ReceivePacket),
    WriteAcknowledgement(ChannelEvents::WriteAcknowledgement),
    AcknowledgePacket(ChannelEvents::AcknowledgePacket),
    TimeoutPacket(ChannelEvents::TimeoutPacket),
    ChannelClosed(ChannelEvents::ChannelClosed),

    Module(ModuleEvent),
    Message(MessageEvent),
}

impl TryFrom<IbcEvent> for abci::Event {
    type Error = Error;

    fn try_from(event: IbcEvent) -> Result<Self, Self::Error> {
        Ok(match event {
            IbcEvent::CreateClient(event) => event.into(),
            IbcEvent::UpdateClient(event) => event.into(),
            IbcEvent::UpgradeClient(event) => event.into(),
            IbcEvent::ClientMisbehaviour(event) => event.into(),
            IbcEvent::OpenInitConnection(event) => event.into(),
            IbcEvent::OpenTryConnection(event) => event.into(),
            IbcEvent::OpenAckConnection(event) => event.into(),
            IbcEvent::OpenConfirmConnection(event) => event.into(),
            IbcEvent::OpenInitChannel(event) => event.into(),
            IbcEvent::OpenTryChannel(event) => event.into(),
            IbcEvent::OpenAckChannel(event) => event.into(),
            IbcEvent::OpenConfirmChannel(event) => event.into(),
            IbcEvent::CloseInitChannel(event) => event.into(),
            IbcEvent::CloseConfirmChannel(event) => event.into(),
            IbcEvent::SendPacket(event) => event.try_into().map_err(Error::Channel)?,
            IbcEvent::ReceivePacket(event) => event.try_into().map_err(Error::Channel)?,
            IbcEvent::WriteAcknowledgement(event) => event.try_into().map_err(Error::Channel)?,
            IbcEvent::AcknowledgePacket(event) => event.try_into().map_err(Error::Channel)?,
            IbcEvent::TimeoutPacket(event) => event.try_into().map_err(Error::Channel)?,
            IbcEvent::ChannelClosed(event) => event.into(),
            IbcEvent::Module(event) => event.try_into()?,
            IbcEvent::Message(event) => abci::Event {
                kind: MESSAGE_EVENT.to_string(),
                attributes: vec![("module", event.module_attribute(), true).into()],
            },
        })
    }
}

impl IbcEvent {
    pub fn event_type(&self) -> &str {
        match self {
            IbcEvent::CreateClient(event) => event.event_type(),
            IbcEvent::UpdateClient(event) => event.event_type(),
            IbcEvent::ClientMisbehaviour(event) => event.event_type(),
            IbcEvent::UpgradeClient(event) => event.event_type(),
            IbcEvent::OpenInitConnection(event) => event.event_type(),
            IbcEvent::OpenTryConnection(event) => event.event_type(),
            IbcEvent::OpenAckConnection(event) => event.event_type(),
            IbcEvent::OpenConfirmConnection(event) => event.event_type(),
            IbcEvent::OpenInitChannel(event) => event.event_type(),
            IbcEvent::OpenTryChannel(event) => event.event_type(),
            IbcEvent::OpenAckChannel(event) => event.event_type(),
            IbcEvent::OpenConfirmChannel(event) => event.event_type(),
            IbcEvent::CloseInitChannel(event) => event.event_type(),
            IbcEvent::CloseConfirmChannel(event) => event.event_type(),
            IbcEvent::SendPacket(event) => event.event_type(),
            IbcEvent::ReceivePacket(event) => event.event_type(),
            IbcEvent::WriteAcknowledgement(event) => event.event_type(),
            IbcEvent::AcknowledgePacket(event) => event.event_type(),
            IbcEvent::TimeoutPacket(event) => event.event_type(),
            IbcEvent::ChannelClosed(event) => event.event_type(),
            IbcEvent::Module(module_event) => module_event.kind.as_str(),
            IbcEvent::Message(_) => MESSAGE_EVENT,
        }
    }
}

/// The event type emitted by IBC applications
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEvent {
    pub kind: String,
    pub attributes: Vec<ModuleEventAttribute>,
}

impl TryFrom<ModuleEvent> for abci::Event {
    type Error = Error;

    fn try_from(event: ModuleEvent) -> Result<Self, Self::Error> {
        let attributes = event.attributes.into_iter().map(Into::into).collect();
        Ok(abci::Event {
            kind: event.kind,
            attributes,
        })
    }
}

impl From<ModuleEvent> for IbcEvent {
    fn from(e: ModuleEvent) -> Self {
        IbcEvent::Module(e)
    }
}

///  A single key/value pair in a [`ModuleEvent`]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEventAttribute {
    pub key: String,
    pub value: String,
}

impl<K: ToString, V: ToString> From<(K, V)> for ModuleEventAttribute {
    fn from((k, v): (K, V)) -> Self {
        Self {
            key: k.to_string(),
            value: v.to_string(),
        }
    }
}

impl From<ModuleEventAttribute> for abci::EventAttribute {
    fn from(attr: ModuleEventAttribute) -> Self {
        (attr.key, attr.value).into()
    }
}

/// An event type that is emitted by the Cosmos SDK.
///
/// We need to emit it as well, as currently [hermes] relies on it.
///
/// [hermes]: https://github.com/informalsystems/hermes
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageEvent {
    Client,
    Connection,
    Channel,
    // stores the module name
    Module(String),
}

impl MessageEvent {
    /// The ABCI event attribute has only one attribute, with key `module`.
    /// This method gets the associated value.
    pub fn module_attribute(&self) -> String {
        match self {
            MessageEvent::Client => "ibc_client".to_string(),
            MessageEvent::Connection => "ibc_connection".to_string(),
            MessageEvent::Channel => "ibc_channel".to_string(),
            MessageEvent::Module(module_name) => module_name.clone(),
        }
    }
}

impl From<MessageEvent> for IbcEvent {
    fn from(e: MessageEvent) -> Self {
        IbcEvent::Message(e)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use alloc::vec;

    use crate::core::{
        ics04_channel::{
            channel::Order,
            events::SendPacket,
            packet::{test_utils::get_dummy_raw_packet, Packet},
        },
        ics24_host::identifier::ConnectionId,
    };

    #[test]
    /// Ensures that we don't panic when packet data is not valid UTF-8.
    /// See issue [#199](https://github.com/cosmos/ibc-rs/issues/199)
    pub fn test_packet_data_non_utf8() {
        let mut packet = Packet::try_from(get_dummy_raw_packet(1, 1)).unwrap();
        packet.data = vec![128];

        let ibc_event = IbcEvent::SendPacket(SendPacket::new(
            packet,
            Order::Unordered,
            ConnectionId::default(),
        ));
        let _ = abci::Event::try_from(ibc_event);
    }
}
