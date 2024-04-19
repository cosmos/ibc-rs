mod codec;

pub use codec::*;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::primitives::proto::Protobuf;

/// Encodes a `Height` into a Protobuf encoded byte array.
pub fn encode_height(height: Height) -> Vec<u8> {
    height.encode_vec()
}

/// Decodes a `Height` from a Protobuf encoded byte array.
pub fn decode_height(encoded_height: Vec<u8>) -> Result<Height, ClientError> {
    Height::decode_vec(&encoded_height).map_err(|e| ClientError::Other {
        description: e.to_string(),
    })
}
