use ibc_primitives::prelude::*;
use tendermint::abci;

/// The event type emitted by IBC applications
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEvent {
    pub kind: String,
    pub attributes: Vec<ModuleEventAttribute>,
}

impl From<ModuleEvent> for abci::Event {
    fn from(event: ModuleEvent) -> Self {
        let attributes = event.attributes.into_iter().map(Into::into).collect();
        abci::Event {
            kind: event.kind,
            attributes,
        }
    }
}

///  A single key/value pair in a [`ModuleEvent`]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEventAttribute {
    pub key: String,
    pub value: String,
}

impl<K: ToString, V: ToString> From<(K, V)> for ModuleEventAttribute {
    fn from((k, v): (K, V)) -> Self {
        Self {
            key: k.to_string(),
            value: v.to_string(),
        }
    }
}

impl From<ModuleEventAttribute> for abci::EventAttribute {
    fn from(attr: ModuleEventAttribute) -> Self {
        (attr.key, attr.value).into()
    }
}
