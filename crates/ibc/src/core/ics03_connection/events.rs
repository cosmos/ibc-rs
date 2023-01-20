//! Types for the IBC events emitted from Tendermint Websocket by the connection module.

use tendermint::abci;

use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::events::IbcEventType;
use crate::prelude::*;

/// The content of the `key` field for the attribute containing the connection identifier.
pub const CONN_ID_ATTRIBUTE_KEY: &str = "connection_id";
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";
pub const COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY: &str = "counterparty_connection_id";
pub const COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY: &str = "counterparty_client_id";

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
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Attributes {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_connection_id: Option<ConnectionId>,
    pub counterparty_client_id: ClientId,
}

/// Convert attributes to Tendermint ABCI tags
impl From<Attributes> for Vec<abci::EventAttribute> {
    fn from(a: Attributes) -> Self {
        let conn_id = (CONN_ID_ATTRIBUTE_KEY, a.connection_id.as_str()).into();
        let client_id = (CLIENT_ID_ATTRIBUTE_KEY, a.client_id.as_str()).into();

        let counterparty_conn_id = (
            COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY,
            a.counterparty_connection_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or(""),
        )
            .into();

        let counterparty_client_id = (
            COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY,
            a.counterparty_client_id.as_str(),
        )
            .into();

        vec![
            conn_id,
            client_id,
            counterparty_client_id,
            counterparty_conn_id,
        ]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenInit(Attributes);

impl OpenInit {
    /// Per our convention, this event is generated on chain A.
    pub fn new(
        conn_id_on_a: ConnectionId,
        client_id_on_a: ClientId,
        client_id_on_b: ClientId,
    ) -> Self {
        Self(Attributes {
            connection_id: conn_id_on_a,
            client_id: client_id_on_a,
            counterparty_connection_id: None,
            counterparty_client_id: client_id_on_b,
        })
    }

    pub fn connection_id(&self) -> &ConnectionId {
        &self.0.connection_id
    }
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
    pub fn counterparty_connection_id(&self) -> Option<&ConnectionId> {
        self.0.counterparty_connection_id.as_ref()
    }
    pub fn counterparty_client_id(&self) -> &ClientId {
        &self.0.counterparty_client_id
    }
}

impl From<OpenInit> for abci::Event {
    fn from(v: OpenInit) -> Self {
        abci::Event {
            kind: IbcEventType::OpenInitConnection.as_str().to_owned(),
            attributes: v.0.into(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTry(Attributes);

impl OpenTry {
    /// Per our convention, this event is generated on chain B.
    pub fn new(
        conn_id_on_b: ConnectionId,
        client_id_on_b: ClientId,
        conn_id_on_a: ConnectionId,
        client_id_on_a: ClientId,
    ) -> Self {
        Self(Attributes {
            connection_id: conn_id_on_b,
            client_id: client_id_on_b,
            counterparty_connection_id: Some(conn_id_on_a),
            counterparty_client_id: client_id_on_a,
        })
    }

    pub fn connection_id(&self) -> &ConnectionId {
        &self.0.connection_id
    }
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
    pub fn counterparty_connection_id(&self) -> Option<&ConnectionId> {
        self.0.counterparty_connection_id.as_ref()
    }
    pub fn counterparty_client_id(&self) -> &ClientId {
        &self.0.counterparty_client_id
    }
}

impl From<OpenTry> for abci::Event {
    fn from(v: OpenTry) -> Self {
        abci::Event {
            kind: IbcEventType::OpenTryConnection.as_str().to_owned(),
            attributes: v.0.into(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAck(Attributes);

impl OpenAck {
    /// Per our convention, this event is generated on chain A.
    pub fn new(
        conn_id_on_a: ConnectionId,
        client_id_on_a: ClientId,
        conn_id_on_b: ConnectionId,
        client_id_on_b: ClientId,
    ) -> Self {
        Self(Attributes {
            connection_id: conn_id_on_a,
            client_id: client_id_on_a,
            counterparty_connection_id: Some(conn_id_on_b),
            counterparty_client_id: client_id_on_b,
        })
    }

    pub fn connection_id(&self) -> &ConnectionId {
        &self.0.connection_id
    }
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
    pub fn counterparty_connection_id(&self) -> Option<&ConnectionId> {
        self.0.counterparty_connection_id.as_ref()
    }
    pub fn counterparty_client_id(&self) -> &ClientId {
        &self.0.counterparty_client_id
    }
}

impl From<OpenAck> for abci::Event {
    fn from(v: OpenAck) -> Self {
        abci::Event {
            kind: IbcEventType::OpenAckConnection.as_str().to_owned(),
            attributes: v.0.into(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenConfirm(Attributes);

impl OpenConfirm {
    /// Per our convention, this event is generated on chain B.
    pub fn new(
        conn_id_on_b: ConnectionId,
        client_id_on_b: ClientId,
        conn_id_on_a: ConnectionId,
        client_id_on_a: ClientId,
    ) -> Self {
        Self(Attributes {
            connection_id: conn_id_on_b,
            client_id: client_id_on_b,
            counterparty_connection_id: Some(conn_id_on_a),
            counterparty_client_id: client_id_on_a,
        })
    }

    pub fn connection_id(&self) -> &ConnectionId {
        &self.0.connection_id
    }
    pub fn client_id(&self) -> &ClientId {
        &self.0.client_id
    }
    pub fn counterparty_connection_id(&self) -> Option<&ConnectionId> {
        self.0.counterparty_connection_id.as_ref()
    }
    pub fn counterparty_client_id(&self) -> &ClientId {
        &self.0.counterparty_client_id
    }
}

impl From<OpenConfirm> for abci::Event {
    fn from(v: OpenConfirm) -> Self {
        abci::Event {
            kind: IbcEventType::OpenConfirmConnection.as_str().to_owned(),
            attributes: v.0.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::client_state::client_type as mock_client_type;
    use tendermint::abci::Event as AbciEvent;

    #[test]
    fn ibc_to_abci_connection_events() {
        struct Test {
            kind: IbcEventType,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
        }

        let conn_id_on_a = ConnectionId::default();
        let client_id_on_a = ClientId::default();
        let conn_id_on_b = ConnectionId::new(1);
        let client_id_on_b = ClientId::new(mock_client_type(), 0).unwrap();
        let expected_keys = vec![
            CONN_ID_ATTRIBUTE_KEY,
            CLIENT_ID_ATTRIBUTE_KEY,
            COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY,
            COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY,
        ];

        let tests: Vec<Test> = vec![
            Test {
                kind: IbcEventType::OpenInitConnection,
                event: OpenInit::new(
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                    client_id_on_b.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
            },
            Test {
                kind: IbcEventType::OpenTryConnection,
                event: OpenTry::new(
                    conn_id_on_b.clone(),
                    client_id_on_b.clone(),
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
            },
            Test {
                kind: IbcEventType::OpenAckConnection,
                event: OpenAck::new(
                    conn_id_on_a.clone(),
                    client_id_on_a.clone(),
                    conn_id_on_b.clone(),
                    client_id_on_b.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
            },
            Test {
                kind: IbcEventType::OpenConfirmConnection,
                event: OpenConfirm::new(conn_id_on_b, client_id_on_b, conn_id_on_a, client_id_on_a)
                    .into(),
                expected_keys: expected_keys.clone(),
            },
        ];

        for t in tests {
            assert_eq!(t.kind.as_str(), t.event.kind);
            assert_eq!(t.expected_keys.len(), t.event.attributes.len());
            for (i, key) in t.expected_keys.iter().enumerate() {
                assert_eq!(t.event.attributes[i].key, *key);
            }
        }
    }
}
