use core::any::TypeId;

use ibc_primitives::prelude::String;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityKey(String);

impl From<TypeId> for CapabilityKey {
    fn from(type_id: TypeId) -> Self {
        Self(std::format!("{:?}", type_id))
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
