//! Types for the IBC events emitted from Tendermint Websocket by the client module.

use core::fmt::{Display, Error as FmtError, Formatter};
use serde_derive::{Deserialize, Serialize};
use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use super::header::Header;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::height::Height;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::{IbcEvent, IbcEventType};
use crate::prelude::*;

/// The content of the `key` field for the attribute containing the client identifier.
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";

/// The content of the `key` field for the attribute containing the client type.
pub const CLIENT_TYPE_ATTRIBUTE_KEY: &str = "client_type";

/// The content of the `key` field for the attribute containing the height.
pub const CONSENSUS_HEIGHT_ATTRIBUTE_KEY: &str = "consensus_height";

/// The content of the `key` field for the header in update client event.
pub const HEADER_ATTRIBUTE_KEY: &str = "header";

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

impl From<ClientId> for ClientIdAttribute {
    fn from(client_id: ClientId) -> Self {
        Self { client_id }
    }
}

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

impl From<ClientType> for ClientTypeAttribute {
    fn from(client_type: ClientType) -> Self {
        Self { client_type }
    }
}

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

impl From<Height> for ConsensusHeightAttribute {
    fn from(consensus_height: Height) -> Self {
        Self { consensus_height }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Attributes {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            client_id: Default::default(),
            client_type: ClientType::Tendermint,
            consensus_height: Height::new(0, 1).unwrap(),
        }
    }
}

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Attributes {{ client_id: {}, client_type: {}, consensus_height: {} }}",
            self.client_id, self.client_type, self.consensus_height
        )
    }
}

/// Convert attributes to Tendermint ABCI tags
///
/// # Note
/// The parsing of `Key`s and `Value`s never fails, because the
/// `FromStr` instance of `tendermint::abci::tag::{Key, Value}`
/// is infallible, even if it is not represented in the error type.
/// Once tendermint-rs improves the API of the `Key` and `Value` types,
/// we will be able to remove the `.parse().unwrap()` calls.
impl From<Attributes> for Vec<Tag> {
    fn from(attrs: Attributes) -> Self {
        let client_id = Tag {
            key: CLIENT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attrs.client_id.to_string().parse().unwrap(),
        };
        let client_type = Tag {
            key: CLIENT_TYPE_ATTRIBUTE_KEY.parse().unwrap(),
            value: attrs.client_type.as_str().parse().unwrap(),
        };
        let consensus_height = Tag {
            key: CONSENSUS_HEIGHT_ATTRIBUTE_KEY.parse().unwrap(),
            value: attrs.consensus_height.to_string().parse().unwrap(),
        };
        vec![client_id, client_type, consensus_height]
    }
}

/// CreateClient event signals the creation of a new on-chain client (IBC client).
#[derive(Debug)]
pub struct CreateClient {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub consensus_height: Height,
}

impl Display for CreateClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "CreateClient {{ {} }}", self)
    }
}

impl From<CreateClient> for IbcEvent {
    fn from(v: CreateClient) -> Self {
        IbcEvent::CreateClient(v)
    }
}

impl From<CreateClient> for AbciEvent {
    fn from(c: CreateClient) -> Self {
        AbciEvent {
            type_str: IbcEventType::CreateClient.as_str().to_string(),
            attributes: vec![
                ClientIdAttribute::from(c.client_id).into(),
                ClientTypeAttribute::from(c.client_type).into(),
                ConsensusHeightAttribute::from(c.consensus_height).into(),
            ],
        }
    }
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdateClient {
    pub common: Attributes,
    pub header: Option<Box<dyn Header>>,
}

impl UpdateClient {
    pub fn client_id(&self) -> &ClientId {
        &self.common.client_id
    }

    pub fn client_type(&self) -> ClientType {
        self.common.client_type
    }

    pub fn consensus_height(&self) -> Height {
        self.common.consensus_height
    }
}

impl Display for UpdateClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        // TODO Display: Check for a solution for Box<dyn Header>
        write!(
            f,
            "UpdateClient {{ common: {}, header: None }}",
            self.common
        )
    }
}

impl From<Attributes> for UpdateClient {
    fn from(attrs: Attributes) -> Self {
        UpdateClient {
            common: attrs,
            header: None,
        }
    }
}

impl From<UpdateClient> for IbcEvent {
    fn from(v: UpdateClient) -> Self {
        IbcEvent::UpdateClient(v)
    }
}

impl From<UpdateClient> for AbciEvent {
    fn from(v: UpdateClient) -> Self {
        let mut attributes = Vec::<Tag>::from(v.common);
        if let Some(h) = v.header {
            let header = Tag {
                key: HEADER_ATTRIBUTE_KEY.parse().unwrap(),
                value: h.encode_to_hex_string().parse().unwrap(),
            };
            attributes.push(header);
        }
        AbciEvent {
            type_str: IbcEventType::UpdateClient.as_str().to_string(),
            attributes,
        }
    }
}

/// ClientMisbehaviour event signals the update of an on-chain client (IBC Client) with evidence of
/// misbehaviour.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ClientMisbehaviour(pub Attributes);

impl ClientMisbehaviour {
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
}

impl Display for ClientMisbehaviour {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "ClientMisbehaviour {{ {} }}", self.0)
    }
}

impl From<Attributes> for ClientMisbehaviour {
    fn from(attrs: Attributes) -> Self {
        ClientMisbehaviour(attrs)
    }
}

impl From<ClientMisbehaviour> for IbcEvent {
    fn from(v: ClientMisbehaviour) -> Self {
        IbcEvent::ClientMisbehaviour(v)
    }
}

impl From<ClientMisbehaviour> for AbciEvent {
    fn from(v: ClientMisbehaviour) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::ClientMisbehaviour.as_str().to_string(),
            attributes,
        }
    }
}

/// Signals a recent upgrade of an on-chain client (IBC Client).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct UpgradeClient(pub Attributes);

impl UpgradeClient {
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
}

impl Display for UpgradeClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "UpgradeClient {{ {} }}", self.0)
    }
}

impl From<Attributes> for UpgradeClient {
    fn from(attrs: Attributes) -> Self {
        UpgradeClient(attrs)
    }
}

impl From<UpgradeClient> for AbciEvent {
    fn from(v: UpgradeClient) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::UpgradeClient.as_str().to_string(),
            attributes,
        }
    }
}
