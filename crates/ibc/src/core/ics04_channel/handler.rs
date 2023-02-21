//! This module implements the processing logic for ICS4 (channel) messages.
use crate::events::ModuleEvent;
use crate::prelude::*;

use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics24_host::identifier::{ChannelId, PortId};

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
pub mod write_acknowledgement;

/// Defines the possible states of a channel identifier in a `ChannelResult`.
#[derive(Clone, Debug)]
pub enum ChannelIdState {
    /// Specifies that the channel handshake handler allocated a new channel identifier. This
    /// happens during the processing of either the `MsgChannelOpenInit` or `MsgChannelOpenTry`.
    Generated,

    /// Specifies that the handler reused a previously-allocated channel identifier.
    Reused,
}

#[derive(Clone, Debug)]
pub struct ChannelResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub channel_id_state: ChannelIdState,
    pub channel_end: ChannelEnd,
}

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
