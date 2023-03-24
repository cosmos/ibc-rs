use crate::prelude::*;

use core::convert::{TryFrom, TryInto};
use core::str::FromStr;
use displaydoc::Display;
use tendermint::abci;

use crate::core::ics02_client::error as client_error;
use crate::core::ics02_client::events::{self as ClientEvents};
use crate::core::ics03_connection::error as connection_error;
use crate::core::ics03_connection::events as ConnectionEvents;
use crate::core::ics04_channel::error as channel_error;
use crate::core::ics04_channel::events as ChannelEvents;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics26_routing::context::ModuleId;
use crate::timestamp::ParseTimestampError;

#[derive(Debug, Display)]
pub enum Error {
    /// error parsing height
    Height,
    /// parse error: `{0}`
    Parse(ValidationError),
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

/// Events whose data is not included in the app state and must be extracted using tendermint RPCs
/// (i.e. /tx_search or /block_search)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum WithBlockDataType {
    CreateClient,
    UpdateClient,
    SendPacket,
    WriteAck,
}

impl WithBlockDataType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            WithBlockDataType::CreateClient => "create_client",
            WithBlockDataType::UpdateClient => "update_client",
            WithBlockDataType::SendPacket => "send_packet",
            WithBlockDataType::WriteAck => "write_acknowledgement",
        }
    }
}

const APP_MODULE_EVENT: &str = "app_module";
const MESSAGE_EVENT: &str = "message";
/// Client event types
const CREATE_CLIENT_EVENT: &str = "create_client";
const UPDATE_CLIENT_EVENT: &str = "update_client";
const CLIENT_MISBEHAVIOUR_EVENT: &str = "client_misbehaviour";
const UPGRADE_CLIENT_EVENT: &str = "upgrade_client";
/// Connection event types
const CONNECTION_INIT_EVENT: &str = "connection_open_init";
const CONNECTION_TRY_EVENT: &str = "connection_open_try";
const CONNECTION_ACK_EVENT: &str = "connection_open_ack";
const CONNECTION_CONFIRM_EVENT: &str = "connection_open_confirm";
/// Channel event types
const CHANNEL_OPEN_INIT_EVENT: &str = "channel_open_init";
const CHANNEL_OPEN_TRY_EVENT: &str = "channel_open_try";
const CHANNEL_OPEN_ACK_EVENT: &str = "channel_open_ack";
const CHANNEL_OPEN_CONFIRM_EVENT: &str = "channel_open_confirm";
const CHANNEL_CLOSE_INIT_EVENT: &str = "channel_close_init";
const CHANNEL_CLOSE_CONFIRM_EVENT: &str = "channel_close_confirm";
/// Packet event types
const SEND_PACKET_EVENT: &str = "send_packet";
const RECEIVE_PACKET_EVENT: &str = "receive_packet";
const WRITE_ACK_EVENT: &str = "write_acknowledgement";
const ACK_PACKET_EVENT: &str = "acknowledge_packet";
const TIMEOUT_EVENT: &str = "timeout_packet";
const CHANNEL_CLOSED_EVENT: &str = "channel_close";

/// Events types
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
pub enum IbcEventType {
    CreateClient,
    UpdateClient,
    UpgradeClient,
    ClientMisbehaviour,
    OpenInitConnection,
    OpenTryConnection,
    OpenAckConnection,
    OpenConfirmConnection,
    OpenInitChannel,
    OpenTryChannel,
    OpenAckChannel,
    OpenConfirmChannel,
    CloseInitChannel,
    CloseConfirmChannel,
    ChannelClosed,
    SendPacket,
    ReceivePacket,
    WriteAck,
    AckPacket,
    Timeout,
    AppModule,
    Message,
}

impl IbcEventType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            IbcEventType::CreateClient => CREATE_CLIENT_EVENT,
            IbcEventType::UpdateClient => UPDATE_CLIENT_EVENT,
            IbcEventType::UpgradeClient => UPGRADE_CLIENT_EVENT,
            IbcEventType::ClientMisbehaviour => CLIENT_MISBEHAVIOUR_EVENT,
            IbcEventType::OpenInitConnection => CONNECTION_INIT_EVENT,
            IbcEventType::OpenTryConnection => CONNECTION_TRY_EVENT,
            IbcEventType::OpenAckConnection => CONNECTION_ACK_EVENT,
            IbcEventType::OpenConfirmConnection => CONNECTION_CONFIRM_EVENT,
            IbcEventType::OpenInitChannel => CHANNEL_OPEN_INIT_EVENT,
            IbcEventType::OpenTryChannel => CHANNEL_OPEN_TRY_EVENT,
            IbcEventType::OpenAckChannel => CHANNEL_OPEN_ACK_EVENT,
            IbcEventType::OpenConfirmChannel => CHANNEL_OPEN_CONFIRM_EVENT,
            IbcEventType::CloseInitChannel => CHANNEL_CLOSE_INIT_EVENT,
            IbcEventType::CloseConfirmChannel => CHANNEL_CLOSE_CONFIRM_EVENT,
            IbcEventType::ChannelClosed => CHANNEL_CLOSED_EVENT,
            IbcEventType::SendPacket => SEND_PACKET_EVENT,
            IbcEventType::ReceivePacket => RECEIVE_PACKET_EVENT,
            IbcEventType::WriteAck => WRITE_ACK_EVENT,
            IbcEventType::AckPacket => ACK_PACKET_EVENT,
            IbcEventType::Timeout => TIMEOUT_EVENT,
            IbcEventType::AppModule => APP_MODULE_EVENT,
            IbcEventType::Message => MESSAGE_EVENT,
        }
    }
}

impl FromStr for IbcEventType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CREATE_CLIENT_EVENT => Ok(IbcEventType::CreateClient),
            UPDATE_CLIENT_EVENT => Ok(IbcEventType::UpdateClient),
            UPGRADE_CLIENT_EVENT => Ok(IbcEventType::UpgradeClient),
            CLIENT_MISBEHAVIOUR_EVENT => Ok(IbcEventType::ClientMisbehaviour),
            CONNECTION_INIT_EVENT => Ok(IbcEventType::OpenInitConnection),
            CONNECTION_TRY_EVENT => Ok(IbcEventType::OpenTryConnection),
            CONNECTION_ACK_EVENT => Ok(IbcEventType::OpenAckConnection),
            CONNECTION_CONFIRM_EVENT => Ok(IbcEventType::OpenConfirmConnection),
            CHANNEL_OPEN_INIT_EVENT => Ok(IbcEventType::OpenInitChannel),
            CHANNEL_OPEN_TRY_EVENT => Ok(IbcEventType::OpenTryChannel),
            CHANNEL_OPEN_ACK_EVENT => Ok(IbcEventType::OpenAckChannel),
            CHANNEL_OPEN_CONFIRM_EVENT => Ok(IbcEventType::OpenConfirmChannel),
            CHANNEL_CLOSE_INIT_EVENT => Ok(IbcEventType::CloseInitChannel),
            CHANNEL_CLOSE_CONFIRM_EVENT => Ok(IbcEventType::CloseConfirmChannel),
            SEND_PACKET_EVENT => Ok(IbcEventType::SendPacket),
            RECEIVE_PACKET_EVENT => Ok(IbcEventType::ReceivePacket),
            WRITE_ACK_EVENT => Ok(IbcEventType::WriteAck),
            ACK_PACKET_EVENT => Ok(IbcEventType::AckPacket),
            TIMEOUT_EVENT => Ok(IbcEventType::Timeout),
            CHANNEL_CLOSED_EVENT => Ok(IbcEventType::ChannelClosed),
            // from_str() for `APP_MODULE_EVENT` MUST fail because a `ModuleEvent`'s type isn't constant
            _ => Err(Error::IncorrectEventType {
                event: s.to_string(),
            }),
        }
    }
}

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

    AppModule(ModuleEvent),

    Message(IbcEventType),
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
            IbcEvent::AppModule(event) => event.try_into()?,
            IbcEvent::Message(event_type) => try_message_event_from_event_type(event_type)?,
        })
    }
}

impl IbcEvent {
    pub fn event_type(&self) -> IbcEventType {
        match self {
            IbcEvent::CreateClient(_) => IbcEventType::CreateClient,
            IbcEvent::UpdateClient(_) => IbcEventType::UpdateClient,
            IbcEvent::ClientMisbehaviour(_) => IbcEventType::ClientMisbehaviour,
            IbcEvent::UpgradeClient(_) => IbcEventType::UpgradeClient,
            IbcEvent::OpenInitConnection(_) => IbcEventType::OpenInitConnection,
            IbcEvent::OpenTryConnection(_) => IbcEventType::OpenTryConnection,
            IbcEvent::OpenAckConnection(_) => IbcEventType::OpenAckConnection,
            IbcEvent::OpenConfirmConnection(_) => IbcEventType::OpenConfirmConnection,
            IbcEvent::OpenInitChannel(_) => IbcEventType::OpenInitChannel,
            IbcEvent::OpenTryChannel(_) => IbcEventType::OpenTryChannel,
            IbcEvent::OpenAckChannel(_) => IbcEventType::OpenAckChannel,
            IbcEvent::OpenConfirmChannel(_) => IbcEventType::OpenConfirmChannel,
            IbcEvent::CloseInitChannel(_) => IbcEventType::CloseInitChannel,
            IbcEvent::CloseConfirmChannel(_) => IbcEventType::CloseConfirmChannel,
            IbcEvent::SendPacket(_) => IbcEventType::SendPacket,
            IbcEvent::ReceivePacket(_) => IbcEventType::ReceivePacket,
            IbcEvent::WriteAcknowledgement(_) => IbcEventType::WriteAck,
            IbcEvent::AcknowledgePacket(_) => IbcEventType::AckPacket,
            IbcEvent::TimeoutPacket(_) => IbcEventType::Timeout,
            IbcEvent::ChannelClosed(_) => IbcEventType::ChannelClosed,
            IbcEvent::AppModule(_) => IbcEventType::AppModule,
            IbcEvent::Message(_) => IbcEventType::Message,
        }
    }
}

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
    pub module_name: ModuleId,
    pub attributes: Vec<ModuleEventAttribute>,
}

impl TryFrom<ModuleEvent> for abci::Event {
    type Error = Error;

    fn try_from(event: ModuleEvent) -> Result<Self, Self::Error> {
        if IbcEventType::from_str(event.kind.as_str()).is_ok() {
            return Err(Error::MalformedModuleEvent { event });
        }

        let attributes = event.attributes.into_iter().map(Into::into).collect();
        Ok(abci::Event {
            kind: event.kind,
            attributes,
        })
    }
}

impl From<ModuleEvent> for IbcEvent {
    fn from(e: ModuleEvent) -> Self {
        IbcEvent::AppModule(e)
    }
}

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

fn try_message_event_from_event_type(event_type: IbcEventType) -> Result<abci::Event, Error> {
    match event_type {
        IbcEventType::CreateClient
        | IbcEventType::UpdateClient
        | IbcEventType::UpgradeClient
        | IbcEventType::ClientMisbehaviour => Ok(abci::Event {
            kind: "message".to_string(),
            attributes: vec![("module", "ibc_client", true).into()],
        }),
        IbcEventType::OpenInitConnection
        | IbcEventType::OpenTryConnection
        | IbcEventType::OpenAckConnection
        | IbcEventType::OpenConfirmConnection => Ok(abci::Event {
            kind: "message".to_string(),
            attributes: vec![("module", "ibc_connection", true).into()],
        }),
        IbcEventType::OpenInitChannel
        | IbcEventType::OpenTryChannel
        | IbcEventType::OpenAckChannel
        | IbcEventType::OpenConfirmChannel
        | IbcEventType::CloseInitChannel
        | IbcEventType::CloseConfirmChannel
        | IbcEventType::SendPacket
        | IbcEventType::ReceivePacket
        | IbcEventType::WriteAck
        | IbcEventType::AckPacket
        | IbcEventType::Timeout
        | IbcEventType::ChannelClosed => Ok(abci::Event {
            kind: "message".to_string(),
            attributes: vec![("module", "ibc_channel", true).into()],
        }),
        _ => Err(Error::IncorrectEventType {
            event: event_type.as_str().to_string(),
        }),
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
