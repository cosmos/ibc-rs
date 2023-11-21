use alloc::borrow::Borrow;
use core::fmt::{Debug, Display, Error as FmtError, Formatter};

use ibc_primitives::prelude::*;

use crate::event::ModuleEvent;

/// Module name, internal to the chain.
///
/// That is, the IBC protocol never exposes this name. Note that this is
/// different from IBC host [identifiers][ibc_core_host_types::identifiers],
/// which are exposed to other chains by the protocol.
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl Borrow<str> for ModuleId {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/// Logs and events produced during module callbacks
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
#[derive(Clone, Debug)]
pub struct ModuleExtras {
    pub events: Vec<ModuleEvent>,
    pub log: Vec<String>,
}

impl ModuleExtras {
    pub fn empty() -> Self {
        ModuleExtras {
            events: Vec::new(),
            log: Vec::new(),
        }
    }
}
