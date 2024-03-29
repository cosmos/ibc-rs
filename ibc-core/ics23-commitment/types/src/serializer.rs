use ibc_primitives::prelude::*;
use serde::ser::{Serialize, Serializer};
use subtle_encoding::{Encoding, Hex};

pub fn ser_hex_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let hex = Hex::upper_case()
        .encode_to_string(data)
        .map_err(|e| serde::ser::Error::custom(format!("failed to serialize hex: {}", e)))?;
    hex.serialize(serializer)
}
