//! Types for the IBC events emitted from Tendermint Websocket by the connection module.

use serde_derive::{Deserialize, Serialize};
use tendermint::abci;

use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::events::IbcEventType;
use crate::prelude::*;

/// The content of the `key` field for the attribute containing the connection identifier.
pub const CONN_ID_ATTRIBUTE_KEY: &str = "connection_id";
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";
pub const COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY: &str = "counterparty_connection_id";
pub const COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY: &str = "counterparty_client_id";

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
