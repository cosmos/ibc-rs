//! Types for the IBC events emitted from Tendermint Websocket by the client module.

use derive_more::From;
use ibc_proto::google::protobuf::Any;
use subtle_encoding::hex;
use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::height::Height;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::IbcEventType;
use crate::prelude::*;

/// The content of the `key` field for the attribute containing the client identifier.
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";

/// The content of the `key` field for the attribute containing the client type.
pub const CLIENT_TYPE_ATTRIBUTE_KEY: &str = "client_type";

/// The content of the `key` field for the attribute containing the height.
pub const CONSENSUS_HEIGHT_ATTRIBUTE_KEY: &str = "consensus_height";

pub const CONSENSUS_HEIGHTS_ATTRIBUTE_KEY: &str = "consensus_heights";

/// The content of the `key` field for the header in update client event.
pub const HEADER_ATTRIBUTE_KEY: &str = "header";

#[derive(Debug, From)]
struct ClientIdAttribute {
    client_id: ClientId,
}

impl From<ClientIdAttribute> for Tag {
    fn from(attr: ClientIdAttribute) -> Self {
        Tag {
            key: CLIENT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.client_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct ClientTypeAttribute {
    client_type: ClientType,
}

impl From<ClientTypeAttribute> for Tag {
    fn from(attr: ClientTypeAttribute) -> Self {
        Tag {
            key: CLIENT_TYPE_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.client_type.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct ConsensusHeightAttribute {
    consensus_height: Height,
}

impl From<ConsensusHeightAttribute> for Tag {
    fn from(attr: ConsensusHeightAttribute) -> Self {
        Tag {
            key: CONSENSUS_HEIGHT_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.consensus_height.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct ConsensusHeightsAttribute {
    consensus_heights: Vec<Height>,
}

impl From<ConsensusHeightsAttribute> for Tag {
    fn from(attr: ConsensusHeightsAttribute) -> Self {
        let consensus_heights: Vec<String> = attr
            .consensus_heights
            .into_iter()
            .map(|consensus_height| consensus_height.to_string())
            .collect();
        Tag {
            key: CONSENSUS_HEIGHTS_ATTRIBUTE_KEY.parse().unwrap(),
            value: consensus_heights.join(",").parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct HeaderAttribute {
    header: Any,
}

impl From<HeaderAttribute> for Tag {
    fn from(attr: HeaderAttribute) -> Self {
        Tag {
            key: HEADER_ATTRIBUTE_KEY.parse().unwrap(),
            value: String::from_utf8(hex::encode(attr.header.value))
                .unwrap()
                .parse()
                .unwrap(),
        }
    }
}

/// CreateClient event signals the creation of a new on-chain client (IBC client).
#[derive(Debug)]
pub struct CreateClient {
    client_id: ClientIdAttribute,
    client_type: ClientTypeAttribute,
    consensus_height: ConsensusHeightAttribute,
}

impl CreateClient {
    pub fn new(client_id: ClientId, client_type: ClientType, consensus_height: Height) -> Self {
        Self {
            client_id: ClientIdAttribute::from(client_id),
            client_type: ClientTypeAttribute::from(client_type),
            consensus_height: ConsensusHeightAttribute::from(consensus_height),
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id.client_id
    }

    pub fn client_type(&self) -> &ClientType {
        &self.client_type.client_type
    }

    pub fn consensus_height(&self) -> &Height {
        &self.consensus_height.consensus_height
    }
}

impl From<CreateClient> for AbciEvent {
    fn from(c: CreateClient) -> Self {
        AbciEvent {
            type_str: IbcEventType::CreateClient.as_str().to_string(),
            attributes: vec![
                c.client_id.into(),
                c.client_type.into(),
                c.consensus_height.into(),
            ],
        }
    }
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
#[derive(Debug)]
pub struct UpdateClient {
    client_id: ClientIdAttribute,
    client_type: ClientTypeAttribute,
    // Deprecated: consensus_height is deprecated and will be removed in a future release.
    // Please use consensus_heights instead.
    consensus_height: ConsensusHeightAttribute,
    consensus_heights: ConsensusHeightsAttribute,
    header: HeaderAttribute,
}

impl UpdateClient {
    pub fn new(
        client_id: ClientId,
        client_type: ClientType,
        consensus_height: Height,
        consensus_heights: Vec<Height>,
        header: Any,
    ) -> Self {
        Self {
            client_id: ClientIdAttribute::from(client_id),
            client_type: ClientTypeAttribute::from(client_type),
            consensus_height: ConsensusHeightAttribute::from(consensus_height),
            consensus_heights: ConsensusHeightsAttribute::from(consensus_heights),
            header: HeaderAttribute::from(header),
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id.client_id
    }

    pub fn client_type(&self) -> &ClientType {
        &self.client_type.client_type
    }

    pub fn consensus_height(&self) -> &Height {
        &self.consensus_height.consensus_height
    }

    pub fn consensus_heights(&self) -> &[Height] {
        self.consensus_heights.consensus_heights.as_ref()
    }

    pub fn header(&self) -> &Any {
        &self.header.header
    }
}

impl From<UpdateClient> for AbciEvent {
    fn from(u: UpdateClient) -> Self {
        AbciEvent {
            type_str: IbcEventType::UpdateClient.as_str().to_string(),
            attributes: vec![
                u.client_id.into(),
                u.client_type.into(),
                u.consensus_height.into(),
                u.consensus_heights.into(),
                u.header.into(),
            ],
        }
    }
}

/// ClientMisbehaviour event signals the update of an on-chain client (IBC Client) with evidence of
/// misbehaviour.
#[derive(Debug)]
pub struct ClientMisbehaviour {
    client_id: ClientIdAttribute,
    client_type: ClientTypeAttribute,
}

impl ClientMisbehaviour {
    pub fn new(client_id: ClientId, client_type: ClientType) -> Self {
        Self {
            client_id: ClientIdAttribute::from(client_id),
            client_type: ClientTypeAttribute::from(client_type),
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id.client_id
    }

    pub fn client_type(&self) -> &ClientType {
        &self.client_type.client_type
    }
}

impl From<ClientMisbehaviour> for AbciEvent {
    fn from(c: ClientMisbehaviour) -> Self {
        AbciEvent {
            type_str: IbcEventType::ClientMisbehaviour.as_str().to_string(),
            attributes: vec![c.client_id.into(), c.client_type.into()],
        }
    }
}

/// Signals a recent upgrade of an on-chain client (IBC Client).
#[derive(Debug)]
pub struct UpgradeClient {
    client_id: ClientIdAttribute,
    client_type: ClientTypeAttribute,
    consensus_height: ConsensusHeightAttribute,
}

impl UpgradeClient {
    pub fn new(client_id: ClientId, client_type: ClientType, consensus_height: Height) -> Self {
        Self {
            client_id: ClientIdAttribute::from(client_id),
            client_type: ClientTypeAttribute::from(client_type),
            consensus_height: ConsensusHeightAttribute::from(consensus_height),
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id.client_id
    }

    pub fn client_type(&self) -> &ClientType {
        &self.client_type.client_type
    }

    pub fn consensus_height(&self) -> &Height {
        &self.consensus_height.consensus_height
    }
}

impl From<UpgradeClient> for AbciEvent {
    fn from(u: UpgradeClient) -> Self {
        AbciEvent {
            type_str: IbcEventType::UpgradeClient.as_str().to_string(),
            attributes: vec![
                u.client_id.into(),
                u.client_type.into(),
                u.consensus_height.into(),
            ],
        }
    }
}
