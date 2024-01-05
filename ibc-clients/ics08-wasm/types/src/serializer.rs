use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ibc_primitives::prelude::*;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Base64;

impl Base64 {
    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = BASE64_STANDARD.encode(bytes);
        String::serialize(&encoded, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(base64.as_bytes())
            .map_err(Error::custom)?;

        Ok(bytes)
    }
}
