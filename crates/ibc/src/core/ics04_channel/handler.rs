//! This module implements the processing logic for ICS4 (channel) messages.
use crate::core::events::ModuleEvent;
use crate::prelude::*;

pub(crate) mod acknowledgement;
pub(crate) mod chan_close_confirm;
pub(crate) mod chan_close_init;
pub(crate) mod chan_open_ack;
pub(crate) mod chan_open_confirm;
pub(crate) mod chan_open_init;
pub(crate) mod chan_open_try;
pub(crate) mod recv_packet;
pub(crate) mod send_packet;
pub(crate) mod timeout;
pub(crate) mod timeout_on_close;

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
