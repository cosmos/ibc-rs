//! Types for the IBC events emitted from Tendermint Websocket by the client module.
use derive_more::From;
use ibc_core_host_types::identifiers::{ClientId, ClientType};
use ibc_primitives::prelude::*;
use self::str::FromStr;
use subtle_encoding::hex;
use tendermint::abci;

use crate::height::Height;
use crate::error::ClientError;
/// Client event types
pub const CREATE_CLIENT_EVENT: &str = "create_client";
pub const UPDATE_CLIENT_EVENT: &str = "update_client";
pub const CLIENT_MISBEHAVIOUR_EVENT: &str = "client_misbehaviour";
pub const UPGRADE_CLIENT_EVENT: &str = "upgrade_client";

/// The content of the `key` field for the attribute containing the client identifier.
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";

/// The content of the `key` field for the attribute containing the client type.
pub const CLIENT_TYPE_ATTRIBUTE_KEY: &str = "client_type";

/// The content of the `key` field for the attribute containing the height.
pub const CONSENSUS_HEIGHT_ATTRIBUTE_KEY: &str = "consensus_height";

/// The content of the `key` field for the attribute containing the heights of consensus states that were processed.
pub const CONSENSUS_HEIGHTS_ATTRIBUTE_KEY: &str = "consensus_heights";

/// The content of the `key` field for the header in update client event.
pub const HEADER_ATTRIBUTE_KEY: &str = "header";

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
struct ClientIdAttribute {
    client_id: ClientId,
}

impl From<ClientIdAttribute> for abci::EventAttribute {
    fn from(attr: ClientIdAttribute) -> Self {
        (CLIENT_ID_ATTRIBUTE_KEY, attr.client_id.as_str()).into()
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
struct ClientTypeAttribute {
    client_type: ClientType,
}

impl From<ClientTypeAttribute> for abci::EventAttribute {
    fn from(attr: ClientTypeAttribute) -> Self {
        (CLIENT_TYPE_ATTRIBUTE_KEY, attr.client_type.as_str()).into()
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
struct ConsensusHeightAttribute {
    consensus_height: Height,
}

impl From<ConsensusHeightAttribute> for abci::EventAttribute {
    fn from(attr: ConsensusHeightAttribute) -> Self {
        (CONSENSUS_HEIGHT_ATTRIBUTE_KEY, attr.consensus_height).into()
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
struct ConsensusHeightsAttribute {
    consensus_heights: Vec<Height>,
}

impl From<ConsensusHeightsAttribute> for abci::EventAttribute {
    fn from(attr: ConsensusHeightsAttribute) -> Self {
        let consensus_heights: Vec<String> = attr
            .consensus_heights
            .into_iter()
            .map(|consensus_height| consensus_height.to_string())
            .collect();
        (CONSENSUS_HEIGHTS_ATTRIBUTE_KEY, consensus_heights.join(",")).into()
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
struct HeaderAttribute {
    /// NOTE: The header is encoded as bytes of the
    /// [`Any`](ibc_proto::google::protobuf::Any) type.
    header: Vec<u8>,
}

impl From<HeaderAttribute> for abci::EventAttribute {
    fn from(attr: HeaderAttribute) -> Self {
        (
            HEADER_ATTRIBUTE_KEY,
            str::from_utf8(&hex::encode(attr.header))
                .expect("Never fails because hexadecimal is valid UTF-8"),
        )
            .into()
    }
}

/// CreateClient event signals the creation of a new on-chain client (IBC client).
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

    pub fn event_type(&self) -> &str {
        CREATE_CLIENT_EVENT
    }
}

impl From<CreateClient> for abci::Event {
    fn from(c: CreateClient) -> Self {
        Self {
            kind: CREATE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                c.client_id.into(),
                c.client_type.into(),
                c.consensus_height.into(),
            ],
        }
    }
}
// impl TryFrom<abci::Event> for CreateClient {
//     type Error = ...;

//     fn try_from(value: abci::Event) -> Result<Self, Self::Error> {
//         (value.kind == CREATE_CLIENT_EVENT)
//             .then(|| { ... })
//             .ok_or_else(|| { ... })
//     }
// }
impl TryFrom<abci::Event> for CreateClient {
    type Error = ClientError;

    fn try_from(value: abci::Event) -> Result<Self, Self::Error> {
        if value.kind != CREATE_CLIENT_EVENT {
            return Err(ClientError::Other {
                description: "Error in parsing CreateClient event".to_string(),
            });
        }

        let mut client_id = None;
        let mut client_type = None;
        let mut consensus_height = None;

        for attribute in value.attributes.iter() {
            let key = attribute.key_str().map_err(|_| ClientError::Other {
                description: "Invalid attribute key".to_string(),
            })?;

            match key {
                CLIENT_ID_ATTRIBUTE_KEY => {
                    client_id = Some(parse_attribute_value(
                        attribute,
                        |value| ClientId::from_str(value).map_err(|_| ClientError::Other {
                            description: "Invalid client ID attribute value".to_string(),
                        }),
                    )?);
                }
                CLIENT_TYPE_ATTRIBUTE_KEY => {
                    client_type = Some(parse_attribute_value(
                        attribute,
                        |value| ClientType::from_str(value).map_err(|_| ClientError::Other {
                            description: "Invalid client type attribute value".to_string(),
                        }),
                    )?);
                }
                CONSENSUS_HEIGHT_ATTRIBUTE_KEY => {
                    consensus_height = Some(parse_attribute_value(
                        attribute,
                        |value| Height::from_str(value).map_err(|_| ClientError::Other {
                            description: "Invalid consensus height attribute value".to_string(),
                        }),
                    )?);
                }
                _ => {}
            }
        }

        let client_id = client_id.ok_or_else(|| ClientError::Other {
            description: "Missing client ID attribute".to_string(),
        })?;
        let client_type = client_type.ok_or_else(|| ClientError::Other {
            description: "Missing client type attribute".to_string(),
        })?;
        let consensus_height = consensus_height.ok_or_else(|| ClientError::Other {
            description: "Missing consensus height attribute".to_string(),
        })?;

        Ok(CreateClient::new(client_id, client_type, consensus_height))
    }
}

fn parse_attribute_value<T>(
    attribute: &abci::EventAttribute,
    f: impl FnOnce(&str) -> Result<T, ClientError>,
) -> Result<T, ClientError> {
    match attribute.value_str() {
        Ok(value) => f(value),
        Err(_) => Err(ClientError::Other {
            description: "Invalid attribute value".to_string(),
        }),
    }
}

/// UpdateClient event signals a recent update of an on-chain client (IBC Client).
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
    /// Constructs a new UpdateClient event.
    ///
    /// NOTE: the `header` is the encoded bytes of the
    /// [`Any`](ibc_proto::google::protobuf::Any) type.
    pub fn new(
        client_id: ClientId,
        client_type: ClientType,
        consensus_height: Height,
        consensus_heights: Vec<Height>,
        header: Vec<u8>,
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

    pub fn header(&self) -> &Vec<u8> {
        &self.header.header
    }

    pub fn event_type(&self) -> &str {
        UPDATE_CLIENT_EVENT
    }
}

impl From<UpdateClient> for abci::Event {
    fn from(u: UpdateClient) -> Self {
        Self {
            kind: UPDATE_CLIENT_EVENT.to_owned(),
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

    pub fn event_type(&self) -> &str {
        CLIENT_MISBEHAVIOUR_EVENT
    }
}

impl From<ClientMisbehaviour> for abci::Event {
    fn from(c: ClientMisbehaviour) -> Self {
        Self {
            kind: CLIENT_MISBEHAVIOUR_EVENT.to_owned(),
            attributes: vec![c.client_id.into(), c.client_type.into()],
        }
    }
}

/// Signals a recent upgrade of an on-chain client (IBC Client).
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

    pub fn event_type(&self) -> &str {
        UPGRADE_CLIENT_EVENT
    }
}

impl From<UpgradeClient> for abci::Event {
    fn from(u: UpgradeClient) -> Self {
        Self {
            kind: UPGRADE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                u.client_id.into(),
                u.client_type.into(),
                u.consensus_height.into(),
            ],
        }
    }
}
