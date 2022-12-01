use abci_tag_proc_marco::abci_tag;
///! This module holds all the abci event attributes for IBC events emitted
///! during the channel handshake.
use derive_more::From;
use tendermint::abci::EventAttribute;

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

#[abci_tag(PORT_ID_ATTRIBUTE_KEY)]
#[derive(Debug, From)]
pub struct PortIdAttribute {
    pub port_id: PortId,
}

#[abci_tag(CHANNEL_ID_ATTRIBUTE_KEY)]
#[derive(Clone, Debug, From)]
pub struct ChannelIdAttribute {
    pub channel_id: ChannelId,
}

#[abci_tag(COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY)]
#[derive(Debug, From)]
pub struct CounterpartyPortIdAttribute {
    pub counterparty_port_id: PortId,
}

#[abci_tag(COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY)]
#[derive(Debug, From)]
pub struct CounterpartyChannelIdAttribute {
    pub counterparty_channel_id: ChannelId,
}

impl AsRef<ChannelId> for CounterpartyChannelIdAttribute {
    fn as_ref(&self) -> &ChannelId {
        &self.counterparty_channel_id
    }
}

#[abci_tag(CONNECTION_ID_ATTRIBUTE_KEY)]
#[derive(Debug, From)]
pub struct ConnectionIdAttribute {
    pub connection_id: ConnectionId,
}

#[abci_tag(VERSION_ATTRIBUTE_KEY)]
#[derive(Debug, From)]
pub struct VersionAttribute {
    pub version: Version,
}
