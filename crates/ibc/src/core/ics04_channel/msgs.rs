//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.


pub(crate) mod chan_open_ack;
pub(crate) mod chan_open_confirm;
pub(crate) mod chan_open_init;
pub(crate) mod chan_open_try;
pub(crate) mod chan_close_confirm;
pub(crate) mod chan_close_init;
pub(crate) mod acknowledgement;
pub(crate) mod recv_packet;
pub(crate) mod timeout;
pub(crate) mod timeout_on_close;

// Opening handshake messages.
pub use chan_open_init::MsgChannelOpenInit;
pub use chan_open_try::MsgChannelOpenTry;
pub use chan_open_ack::MsgChannelOpenAck;
pub use chan_open_confirm::MsgChannelOpenConfirm;

// Closing handshake messages.
pub use chan_close_init::MsgChannelCloseInit;
pub use chan_close_confirm::MsgChannelCloseConfirm;

// Packet specific messages.
pub use recv_packet::MsgRecvPacket;
pub use acknowledgement::MsgAcknowledgement;
pub use timeout::MsgTimeout;
pub use timeout_on_close::MsgTimeoutOnClose;

/// All channel messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelMsg {
    OpenInit(MsgChannelOpenInit),
    OpenTry(MsgChannelOpenTry),
    OpenAck(MsgChannelOpenAck),
    OpenConfirm(MsgChannelOpenConfirm),
    CloseInit(MsgChannelCloseInit),
    CloseConfirm(MsgChannelCloseConfirm),
}

/// All packet messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketMsg {
    Recv(MsgRecvPacket),
    Ack(MsgAcknowledgement),
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}
