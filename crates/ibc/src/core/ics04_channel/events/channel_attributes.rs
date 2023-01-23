///! This module holds all the abci event attributes for IBC events emitted
///! during the channel handshake.
use derive_more::From;
use tendermint::abci;

use crate::core::{
    ics04_channel::Version,
    ics24_host::identifier::{ChannelId, ConnectionId, PortId},
};

const CONNECTION_ID_ATTRIBUTE_KEY: &str = "connection_id";
const CHANNEL_ID_ATTRIBUTE_KEY: &str = "channel_id";
const PORT_ID_ATTRIBUTE_KEY: &str = "port_id";
/// This attribute key is public so that OpenInit can use it to convert itself
/// to an `AbciEvent`
pub(super) const COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY: &str = "counterparty_channel_id";
const COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY: &str = "counterparty_port_id";
const VERSION_ATTRIBUTE_KEY: &str = "version";

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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct PortIdAttribute {
    pub port_id: PortId,
}

impl From<PortIdAttribute> for abci::EventAttribute {
    fn from(attr: PortIdAttribute) -> Self {
        (PORT_ID_ATTRIBUTE_KEY, attr.port_id.as_str()).into()
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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct ChannelIdAttribute {
    pub channel_id: ChannelId,
}

impl From<ChannelIdAttribute> for abci::EventAttribute {
    fn from(attr: ChannelIdAttribute) -> Self {
        (CHANNEL_ID_ATTRIBUTE_KEY, attr.channel_id.as_str()).into()
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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct CounterpartyPortIdAttribute {
    pub counterparty_port_id: PortId,
}

impl From<CounterpartyPortIdAttribute> for abci::EventAttribute {
    fn from(attr: CounterpartyPortIdAttribute) -> Self {
        (
            COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY,
            attr.counterparty_port_id.as_str(),
        )
            .into()
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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct CounterpartyChannelIdAttribute {
    pub counterparty_channel_id: ChannelId,
}

impl From<CounterpartyChannelIdAttribute> for abci::EventAttribute {
    fn from(attr: CounterpartyChannelIdAttribute) -> Self {
        (
            COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY,
            attr.counterparty_channel_id.as_str(),
        )
            .into()
    }
}

impl AsRef<ChannelId> for CounterpartyChannelIdAttribute {
    fn as_ref(&self) -> &ChannelId {
        &self.counterparty_channel_id
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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct ConnectionIdAttribute {
    pub connection_id: ConnectionId,
}

impl From<ConnectionIdAttribute> for abci::EventAttribute {
    fn from(attr: ConnectionIdAttribute) -> Self {
        (CONNECTION_ID_ATTRIBUTE_KEY, attr.connection_id.as_str()).into()
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
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct VersionAttribute {
    pub version: Version,
}

impl From<VersionAttribute> for abci::EventAttribute {
    fn from(attr: VersionAttribute) -> Self {
        (VERSION_ATTRIBUTE_KEY, attr.version.as_str()).into()
    }
}
