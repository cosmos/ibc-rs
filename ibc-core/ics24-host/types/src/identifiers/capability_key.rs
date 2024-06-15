use core::any::TypeId;
use core::fmt::Write;

use ibc_primitives::prelude::String;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityKey(String);

impl From<TypeId> for CapabilityKey {
    fn from(type_id: TypeId) -> Self {
        let mut buf = String::new();
        write!(buf, "{:?}", type_id).unwrap();
        Self(buf)
    }
}

impl From<String> for CapabilityKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for CapabilityKey {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
