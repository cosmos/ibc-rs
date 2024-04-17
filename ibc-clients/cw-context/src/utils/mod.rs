mod codec;

pub use codec::*;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::{Height, HeightError};

/// Decodes a `Height` from a UTF-8 encoded byte array.
pub fn parse_height(encoded_height: Vec<u8>) -> Result<Option<Height>, ClientError> {
    let height_str = match alloc::str::from_utf8(encoded_height.as_slice()) {
        Ok(s) => s,
        // In cases where the height is unavailable, the encoded representation
        // might not be valid UTF-8, resulting in an invalid string. In such
        // instances, we return None.
        Err(_) => return Ok(None),
    };
    match Height::try_from(height_str) {
        Ok(height) => Ok(Some(height)),
        // This is a valid case, as the key may contain other data. We just skip
        // it.
        Err(HeightError::InvalidFormat { .. }) => Ok(None),
        Err(e) => Err(ClientError::Other {
            description: e.to_string(),
        }),
    }
}
