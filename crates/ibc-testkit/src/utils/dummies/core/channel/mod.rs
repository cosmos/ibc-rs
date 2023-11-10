mod acknowledgement;
mod chan_close_confirm;
mod chan_close_init;
mod chan_open_ack;
mod chan_open_confirm;
mod chan_open_init;
mod chan_open_try;
mod packet;
mod recv_packet;
mod timeout;
mod timeout_on_close;

pub use acknowledgement::*;
pub use chan_close_confirm::*;
pub use chan_close_init::*;
pub use chan_open_ack::*;
pub use chan_open_confirm::*;
pub use chan_open_init::*;
pub use chan_open_try::*;
use ibc::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use ibc::prelude::*;
use ibc::proto::core::channel::v1::{Channel as RawChannel, Counterparty as RawCounterparty};
pub use packet::*;
pub use recv_packet::*;
pub use timeout::*;
pub use timeout_on_close::*;

/// Returns a dummy `RawCounterparty`, for testing only!
/// Can be optionally parametrized with a specific channel identifier.
pub fn dummy_raw_counterparty(channel_id: String) -> RawCounterparty {
    RawCounterparty {
        port_id: PortId::transfer().to_string(),
        channel_id,
    }
}

/// Returns a dummy `RawChannel`, for testing only!
pub fn dummy_raw_channel_end(state: i32, channel_id: Option<u64>) -> RawChannel {
    let channel_id = match channel_id {
        Some(id) => ChannelId::new(id).to_string(),
        None => "".to_string(),
    };
    RawChannel {
        state,
        ordering: 2,
        counterparty: Some(dummy_raw_counterparty(channel_id)),
        connection_hops: vec![ConnectionId::default().to_string()],
        version: "".to_string(), // The version is not validated.
    }
}
