//! Types for the IBC events emitted from Tendermint Websocket by the client module.

use derive_more::From;
use ibc_core_host_types::error::DecodingError;
use ibc_core_host_types::identifiers::{ClientId, ClientType};
use ibc_primitives::prelude::*;
use subtle_encoding::hex;
use tendermint::abci;

use self::str::FromStr;
use crate::error::ClientError;
use crate::height::Height;
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
impl TryFrom<abci::EventAttribute> for ClientIdAttribute {
    type Error = DecodingError;

    fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
        if let Ok(key_str) = value.key_str() {
            if key_str != CLIENT_ID_ATTRIBUTE_KEY {
                return Err(DecodingError::InvalidRawData {
                    description: format!("invalid attribute key {}", key_str),
                });
            }
        } else {
            return Err(DecodingError::MissingRawData {
                description: "missing attribute key".to_string(),
            });
        }

        value
            .value_str()
            .map(|value| {
                let client_id = ClientId::from_str(value)?;
                Ok(ClientIdAttribute { client_id })
            })
            .map_err(|e| DecodingError::MissingRawData {
                description: format!("missing attribute value: {e}"),
            })?
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

impl TryFrom<abci::EventAttribute> for ClientTypeAttribute {
    type Error = DecodingError;

    fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
        if let Ok(key_str) = value.key_str() {
            if key_str != CLIENT_TYPE_ATTRIBUTE_KEY {
                return Err(DecodingError::InvalidRawData {
                    description: format!("invalid attribute key {}", key_str),
                });
            }
        } else {
            return Err(DecodingError::MissingRawData {
                description: "missing attribute key".to_string(),
            });
        }

        value
            .value_str()
            .map(|value| {
                let client_type = ClientType::from_str(value)?;
                Ok(ClientTypeAttribute { client_type })
            })
            .map_err(|e| DecodingError::MissingRawData {
                description: format!("missing attribute value: {e}"),
            })?
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

impl TryFrom<abci::EventAttribute> for ConsensusHeightAttribute {
    type Error = DecodingError;

    fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
        if let Ok(key_str) = value.key_str() {
            if key_str != CONSENSUS_HEIGHT_ATTRIBUTE_KEY {
                return Err(DecodingError::InvalidRawData {
                    description: format!("invalid attribute key {}", key_str),
                });
            }
        } else {
            return Err(DecodingError::MissingRawData {
                description: "missing attribute key".to_string(),
            });
        }

        value
            .value_str()
            .map(|value| {
                let consensus_height =
                    Height::from_str(value).map_err(|e| DecodingError::InvalidRawData {
                        description: format!("invalid attribute value: {e}"),
                    })?;
                Ok(ConsensusHeightAttribute { consensus_height })
            })
            .map_err(|e| DecodingError::MissingRawData {
                description: format!("missing attribute value: {e}"),
            })?
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

impl TryFrom<abci::EventAttribute> for ConsensusHeightsAttribute {
    type Error = ClientError;

    fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
        if let Ok(key_str) = value.key_str() {
            if key_str != CONSENSUS_HEIGHTS_ATTRIBUTE_KEY {
                return Err(ClientError::InvalidAttributeKey(key_str.to_string()));
            }
        } else {
            return Err(ClientError::MissingAttributeKey);
        }

        value
            .value_str()
            .map(|value| {
                let consensus_heights: Vec<Height> = value
                    .split(',')
                    .map(|height_str| {
                        Height::from_str(height_str)
                            .map_err(|_| ClientError::InvalidAttributeValue(height_str.to_string()))
                    })
                    .collect::<Result<Vec<Height>, ClientError>>()?;

                Ok(ConsensusHeightsAttribute { consensus_heights })
            })
            .map_err(|_| ClientError::MissingAttributeValue)?
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
impl TryFrom<abci::EventAttribute> for HeaderAttribute {
    type Error = ClientError;

    fn try_from(value: abci::EventAttribute) -> Result<Self, Self::Error> {
        if let Ok(key_str) = value.key_str() {
            if key_str != HEADER_ATTRIBUTE_KEY {
                return Err(ClientError::InvalidAttributeKey(key_str.to_string()));
            }
        } else {
            return Err(ClientError::MissingAttributeKey);
        }

        value
            .value_str()
            .map(|value| {
                let header = hex::decode(value)
                    .map_err(|_| ClientError::InvalidAttributeValue(value.to_string()))?;

                Ok(HeaderAttribute { header })
            })
            .map_err(|_| ClientError::MissingAttributeValue)?
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

impl TryFrom<abci::Event> for CreateClient {
    type Error = DecodingError;

    fn try_from(value: abci::Event) -> Result<Self, Self::Error> {
        if value.kind != CREATE_CLIENT_EVENT {
            return Err(DecodingError::InvalidRawData {
                description: format!("invalid event kind: expected `{}`", CREATE_CLIENT_EVENT),
            });
        }

        value
            .attributes
            .iter()
            .try_fold(
                (None, None, None),
                |(client_id, client_type, consensus_height): (
                    Option<ClientIdAttribute>,
                    Option<ClientTypeAttribute>,
                    Option<ConsensusHeightAttribute>,
                ),
                 attribute| {
                    let key = attribute
                        .key_str()
                        .map_err(|_| DecodingError::MissingRawData {
                            description: "missing attribute key".to_string(),
                        })?;

                    match key {
                        CLIENT_ID_ATTRIBUTE_KEY => Ok((
                            Some(attribute.clone().try_into().map_err(|e| {
                                DecodingError::InvalidRawData {
                                    description: format!("{e}"),
                                }
                            })?),
                            client_type,
                            consensus_height,
                        )),
                        CLIENT_TYPE_ATTRIBUTE_KEY => Ok((
                            client_id,
                            Some(attribute.clone().try_into()?),
                            consensus_height,
                        )),
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY => {
                            Ok((client_id, client_type, Some(attribute.clone().try_into()?)))
                        }
                        _ => Ok((client_id, client_type, consensus_height)),
                    }
                },
            )
            .and_then(
                |(client_id, client_type, consensus_height): (
                    Option<ClientIdAttribute>,
                    Option<ClientTypeAttribute>,
                    Option<ConsensusHeightAttribute>,
                )| {
                    let client_id = client_id.ok_or(DecodingError::MissingRawData {
                        description: "missing attribute key".to_string(),
                    })?;
                    let client_type = client_type.ok_or(DecodingError::MissingRawData {
                        description: "missing attribute key".to_string(),
                    })?;
                    let consensus_height =
                        consensus_height.ok_or(DecodingError::MissingRawData {
                            description: "missing attribute key".to_string(),
                        })?;

                    Ok(CreateClient::new(
                        client_id.client_id,
                        client_type.client_type,
                        consensus_height.consensus_height,
                    ))
                },
            )
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
impl TryFrom<abci::Event> for UpdateClient {
    type Error = ClientError;

    fn try_from(value: abci::Event) -> Result<Self, Self::Error> {
        if value.kind != UPDATE_CLIENT_EVENT {
            return Err(ClientError::Other {
                description: "Error in parsing UpdateClient event".to_string(),
            });
        }

        type UpdateClientAttributes = (
            Option<ClientIdAttribute>,
            Option<ClientTypeAttribute>,
            Option<ConsensusHeightAttribute>,
            Option<ConsensusHeightsAttribute>,
            Option<HeaderAttribute>,
        );

        value
            .attributes
            .iter()
            .try_fold(
                (None, None, None, None, None),
                |acc: UpdateClientAttributes, attribute| {
                    let key = attribute
                        .key_str()
                        .map_err(|_| ClientError::MissingAttributeKey)?;

                    match key {
                        CLIENT_ID_ATTRIBUTE_KEY => Ok((
                            Some(attribute.clone().try_into()?),
                            acc.1,
                            acc.2,
                            acc.3,
                            acc.4,
                        )),
                        CLIENT_TYPE_ATTRIBUTE_KEY => Ok((
                            acc.0,
                            Some(attribute.clone().try_into()?),
                            acc.2,
                            acc.3,
                            acc.4,
                        )),
                        CONSENSUS_HEIGHT_ATTRIBUTE_KEY => Ok((
                            acc.0,
                            acc.1,
                            Some(attribute.clone().try_into()?),
                            acc.3,
                            acc.4,
                        )),
                        CONSENSUS_HEIGHTS_ATTRIBUTE_KEY => Ok((
                            acc.0,
                            acc.1,
                            acc.2,
                            Some(attribute.clone().try_into()?),
                            acc.4,
                        )),
                        HEADER_ATTRIBUTE_KEY => Ok((
                            acc.0,
                            acc.1,
                            acc.2,
                            acc.3,
                            Some(attribute.clone().try_into()?),
                        )),
                        _ => Ok(acc),
                    }
                },
            )
            .and_then(
                |(client_id, client_type, consensus_height, consensus_heights, header)| {
                    let client_id = client_id.ok_or(ClientError::MissingAttributeKey)?.client_id;
                    let client_type = client_type
                        .ok_or(ClientError::MissingAttributeKey)?
                        .client_type;
                    let consensus_height = consensus_height
                        .ok_or(ClientError::MissingAttributeKey)?
                        .consensus_height;
                    let consensus_heights = consensus_heights
                        .ok_or(ClientError::MissingAttributeKey)?
                        .consensus_heights;
                    let header = header.ok_or(ClientError::MissingAttributeKey)?.header;

                    Ok(UpdateClient::new(
                        client_id,
                        client_type,
                        consensus_height,
                        consensus_heights,
                        header,
                    ))
                },
            )
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

#[cfg(test)]
mod tests {
    use core::any::Any;

    use ibc_core_host_types::error::IdentifierError;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(
        abci::Event {
            kind: CREATE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_id", "07-tendermint-0")),
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Ok(CreateClient::new(
            ClientId::from_str("07-tendermint-0").expect("should parse"),
            ClientType::from_str("07-tendermint").expect("should parse"),
            Height::new(1, 10).unwrap(),
        )),
    )]
    #[case(
        abci::Event {
            kind: "some_other_event".to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_id", "07-tendermint-0")),
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Err(IdentifierError::FailedToParse {
            value: "CreateClient".to_string(),
            description: "failed to parse event".to_string()
        }.into())
    )]
    #[case(
        abci::Event {
            kind: CREATE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
            ],
        },
        Err(DecodingError::MissingRawData { description: "missing attribute key".to_string() }),
    )]
    fn test_create_client_try_from(
        #[case] event: abci::Event,
        #[case] expected: Result<CreateClient, DecodingError>,
    ) {
        let result = CreateClient::try_from(event);
        if expected.is_err() {
            assert_eq!(
                result.unwrap_err().type_id(),
                expected.unwrap_err().type_id()
            );
        } else {
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }

    #[rstest]
    #[case(
        abci::Event {
            kind: UPDATE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_id", "07-tendermint-0")),
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
                abci::EventAttribute::from(("consensus_heights", "1-10,1-11")),
                abci::EventAttribute::from(("header", "1234")),
            ],
        },
        Ok(UpdateClient::new(
            ClientId::from_str("07-tendermint-0").expect("should parse"),
            ClientType::from_str("07-tendermint").expect("should parse"),
            Height::new(1, 10).unwrap(),
            vec![Height::new(1, 10).unwrap(), Height::new(1, 11).unwrap()],
            vec![0x12, 0x34],
        )),
    )]
    #[case(
        abci::Event {
            kind: "some_other_event".to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_id", "07-tendermint-0")),
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
                abci::EventAttribute::from(("consensus_heights", "1-10,1-11")),
                abci::EventAttribute::from(("header", "1234")),
            ],
        },
        Err(ClientError::Other {
            description: "Error in parsing UpdateClient event".to_string(),
        }),
    )]
    #[case(
        abci::Event {
            kind: UPDATE_CLIENT_EVENT.to_owned(),
            attributes: vec![
                abci::EventAttribute::from(("client_type", "07-tendermint")),
                abci::EventAttribute::from(("consensus_height", "1-10")),
                abci::EventAttribute::from(("consensus_heights", "1-10,1-11")),
                abci::EventAttribute::from(("header", "1234")),
            ],
        },
        Err(ClientError::MissingAttributeKey),
    )]
    fn test_update_client_try_from(
        #[case] event: abci::Event,
        #[case] expected: Result<UpdateClient, ClientError>,
    ) {
        let result = UpdateClient::try_from(event);
        if expected.is_err() {
            assert_eq!(
                result.unwrap_err().type_id(),
                expected.unwrap_err().type_id()
            );
        } else {
            assert_eq!(result.unwrap(), expected.unwrap());
        }
    }
}
