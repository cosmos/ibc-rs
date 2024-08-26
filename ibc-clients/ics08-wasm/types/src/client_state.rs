//! Defines the client state type for the ICS-08 Wasm light client.

use ibc_core_client::types::Height;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::wasm::v1::ClientState as RawClientState;

use crate::error::WasmClientError;
#[cfg(feature = "serde")]
use crate::serializer::Base64;
use crate::Bytes;

pub const WASM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ClientState";

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "Base64", default))]
    pub data: Bytes,
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "Base64", default))]
    pub checksum: Bytes,
    pub latest_height: Height,
}

impl Protobuf<RawClientState> for ClientState {}

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        Self {
            data: value.data,
            checksum: value.checksum,
            latest_height: Some(value.latest_height.into()),
        }
    }
}

impl TryFrom<RawClientState> for ClientState {
    type Error = WasmClientError;

    fn try_from(raw: RawClientState) -> Result<Self, Self::Error> {
        let latest_height = raw
            .latest_height
            .ok_or(WasmClientError::MissingLatestHeight)?
            .try_into()
            .map_err(|_| WasmClientError::InvalidLatestHeight)?;
        Ok(Self {
            data: raw.data,
            checksum: raw.checksum,
            latest_height,
        })
    }
}

impl Protobuf<Any> for ClientState {}

impl From<ClientState> for Any {
    fn from(value: ClientState) -> Self {
        Self {
            type_url: WASM_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawClientState>::encode_vec(value),
        }
    }
}

impl TryFrom<Any> for ClientState {
    type Error = WasmClientError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        fn decode_client_state(value: &[u8]) -> Result<ClientState, WasmClientError> {
            let client_state = Protobuf::<RawClientState>::decode(value).map_err(|e| {
                WasmClientError::DecodingError {
                    description: e.to_string(),
                }
            })?;

            Ok(client_state)
        }

        match any.type_url.as_str() {
            WASM_CLIENT_STATE_TYPE_URL => decode_client_state(&any.value),
            other_type_url => Err(WasmClientError::MismatchedTypeUrls {
                expected: WASM_CLIENT_STATE_TYPE_URL.to_string(),
                actual: other_type_url.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(b"data", b"checksum", 1)]
    fn test_roundtrip(#[case] data: &[u8], #[case] checksum: &[u8], #[case] height: u64) {
        let raw_client_state = RawClientState {
            data: data.to_vec(),
            checksum: checksum.to_vec(),
            latest_height: Some(RawHeight {
                revision_number: 0,
                revision_height: height,
            }),
        };
        let client_state: ClientState = raw_client_state.clone().try_into().unwrap();
        assert_eq!(RawClientState::from(client_state.clone()), raw_client_state);
        assert_eq!(
            ClientState::try_from(Any::from(client_state.clone())).unwrap(),
            client_state
        );
    }
}
