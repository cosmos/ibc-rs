///! This module holds all the abci event attributes for IBC events emitted
///! during the channel handshake.
use alloc::string::ToString;
use derive_more::From;
use tendermint::abci::tag::Tag;

use crate::core::{
    ics04_channel::Version,
    ics24_host::identifier::{ChannelId, ConnectionId, PortId},
};

const CONNECTION_ID_ATTRIBUTE_KEY: &str = "connection_id";
const CHANNEL_ID_ATTRIBUTE_KEY: &str = "channel_id";
const PORT_ID_ATTRIBUTE_KEY: &str = "port_id";
/// This attribute key is public so that OpenInit can use it to convert itself
/// to an `AbciEvent`
pub const COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY: &str = "counterparty_channel_id";
const COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY: &str = "counterparty_port_id";
const VERSION_ATTRIBUTE_KEY: &str = "version";

#[derive(Debug, From)]
pub struct PortIdAttribute {
    pub port_id: PortId,
}

impl From<PortIdAttribute> for Tag {
    fn from(attr: PortIdAttribute) -> Self {
        Tag {
            key: PORT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.port_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Clone, Debug, From)]
pub struct ChannelIdAttribute {
    pub channel_id: ChannelId,
}

impl From<ChannelIdAttribute> for Tag {
    fn from(attr: ChannelIdAttribute) -> Self {
        Tag {
            key: CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.channel_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct CounterpartyPortIdAttribute {
    pub counterparty_port_id: PortId,
}

impl From<CounterpartyPortIdAttribute> for Tag {
    fn from(attr: CounterpartyPortIdAttribute) -> Self {
        Tag {
            key: COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.counterparty_port_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct CounterpartyChannelIdAttribute {
    pub counterparty_channel_id: ChannelId,
}

impl From<CounterpartyChannelIdAttribute> for Tag {
    fn from(attr: CounterpartyChannelIdAttribute) -> Self {
        Tag {
            key: COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.counterparty_channel_id.to_string().parse().unwrap(),
        }
    }
}

impl AsRef<ChannelId> for CounterpartyChannelIdAttribute {
    fn as_ref(&self) -> &ChannelId {
        &self.counterparty_channel_id
    }
}

#[derive(Debug, From)]
pub struct ConnectionIdAttribute {
    pub connection_id: ConnectionId,
}

impl From<ConnectionIdAttribute> for Tag {
    fn from(attr: ConnectionIdAttribute) -> Self {
        Tag {
            key: CONNECTION_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.connection_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct VersionAttribute {
    pub version: Version,
}

impl From<VersionAttribute> for Tag {
    fn from(attr: VersionAttribute) -> Self {
        Tag {
            key: VERSION_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.version.to_string().parse().unwrap(),
        }
    }
}
