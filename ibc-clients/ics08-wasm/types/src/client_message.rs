//! Defines the client message type for the ICS-08 Wasm light client.

use cosmwasm_std::Binary;
use ibc_primitives::proto::Protobuf;
use ibc_proto::ibc::lightclients::wasm::v1::ClientMessage as RawClientMessage;

pub const WASM_CLIENT_MESSAGE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientMessage";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientMessage {
    pub data: Binary,
}

impl Protobuf<RawClientMessage> for ClientMessage {}

impl From<RawClientMessage> for ClientMessage {
    fn from(raw: RawClientMessage) -> Self {
        Self {
            data: raw.data.into(),
        }
    }
}

impl From<ClientMessage> for RawClientMessage {
    fn from(value: ClientMessage) -> Self {
        RawClientMessage {
            data: value.data.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(b"data")]
    fn test_roundtrip(#[case] data: &[u8]) {
        let raw_msg = RawClientMessage {
            data: data.to_vec(),
        };
        assert_eq!(
            RawClientMessage::from(ClientMessage::from(raw_msg.clone())),
            raw_msg,
        )
    }
}
