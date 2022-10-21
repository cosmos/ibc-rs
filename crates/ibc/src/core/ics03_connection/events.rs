//! Types for the IBC events emitted from Tendermint Websocket by the connection module.

use serde_derive::{Deserialize, Serialize};
use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::events::{IbcEvent, IbcEventType};
use crate::prelude::*;

/// The content of the `key` field for the attribute containing the connection identifier.
pub const CONN_ID_ATTRIBUTE_KEY: &str = "connection_id";
pub const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";
pub const COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY: &str = "counterparty_connection_id";
pub const COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY: &str = "counterparty_client_id";

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Attributes {
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_connection_id: Option<ConnectionId>,
    pub counterparty_client_id: ClientId,
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
    fn from(a: Attributes) -> Self {
        let conn_id = Tag {
            key: CONN_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: a.connection_id.to_string().parse().unwrap(),
        };

        let client_id = Tag {
            key: CLIENT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: a.client_id.to_string().parse().unwrap(),
        };

        let counterparty_conn_id = Tag {
            key: COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: match a.counterparty_connection_id {
                Some(counterparty_conn_id) => counterparty_conn_id.to_string().parse().unwrap(),
                None => "".to_string().parse().unwrap(),
            },
        };

        let counterparty_client_id = Tag {
            key: COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: a.counterparty_client_id.to_string().parse().unwrap(),
        };

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
    pub fn new(
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_connection_id: Option<ConnectionId>,
        counterparty_client_id: ClientId,
    ) -> Self {
        Self(Attributes {
            connection_id,
            client_id,
            counterparty_connection_id,
            counterparty_client_id,
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

impl From<OpenInit> for AbciEvent {
    fn from(v: OpenInit) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::OpenInitConnection.as_str().to_string(),
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OpenTry(pub Attributes);

impl OpenTry {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.0.connection_id.as_ref()
    }
}

impl From<Attributes> for OpenTry {
    fn from(attrs: Attributes) -> Self {
        OpenTry(attrs)
    }
}

impl From<OpenTry> for IbcEvent {
    fn from(v: OpenTry) -> Self {
        IbcEvent::OpenTryConnection(v)
    }
}

impl From<OpenTry> for AbciEvent {
    fn from(v: OpenTry) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::OpenTryConnection.as_str().to_string(),
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OpenAck(pub Attributes);

impl OpenAck {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.0.connection_id.as_ref()
    }
}

impl From<Attributes> for OpenAck {
    fn from(attrs: Attributes) -> Self {
        OpenAck(attrs)
    }
}

impl From<OpenAck> for IbcEvent {
    fn from(v: OpenAck) -> Self {
        IbcEvent::OpenAckConnection(v)
    }
}

impl From<OpenAck> for AbciEvent {
    fn from(v: OpenAck) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::OpenAckConnection.as_str().to_string(),
            attributes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OpenConfirm(pub Attributes);

impl OpenConfirm {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.0.connection_id.as_ref()
    }
}

impl From<Attributes> for OpenConfirm {
    fn from(attrs: Attributes) -> Self {
        OpenConfirm(attrs)
    }
}

impl From<OpenConfirm> for IbcEvent {
    fn from(v: OpenConfirm) -> Self {
        IbcEvent::OpenConfirmConnection(v)
    }
}

impl From<OpenConfirm> for AbciEvent {
    fn from(v: OpenConfirm) -> Self {
        let attributes = Vec::<Tag>::from(v.0);
        AbciEvent {
            type_str: IbcEventType::OpenConfirmConnection.as_str().to_string(),
            attributes,
        }
    }
}
