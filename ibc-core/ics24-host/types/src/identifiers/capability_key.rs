use core::any::TypeId;
use core::fmt::Write;

use ibc_primitives::prelude::*;

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityKey(Vec<u8>);

impl From<TypeId> for CapabilityKey {
    fn from(type_id: TypeId) -> Self {
        let mut buf = String::new();
        write!(buf, "{:?}", type_id).unwrap();
        buf.into()
    }
}

impl From<String> for CapabilityKey {
    fn from(value: String) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl AsRef<[u8]> for CapabilityKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}
