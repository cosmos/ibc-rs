//! This module implements the processing logic for ICS4 (channel) messages.
use crate::events::ModuleEvent;
use crate::prelude::*;

pub mod acknowledgement;
pub mod chan_close_confirm;
pub mod chan_close_init;
pub mod chan_open_ack;
pub mod chan_open_confirm;
pub mod chan_open_init;
pub mod chan_open_try;
pub mod recv_packet;
pub mod send_packet;
pub mod timeout;
pub mod timeout_on_close;

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
