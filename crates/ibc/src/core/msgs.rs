use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use ibc_proto::protobuf::Protobuf;

use crate::core::context::RouterError;
use crate::core::ics02_client::msgs::{
    create_client, misbehaviour, update_client, upgrade_client, ClientMsg,
};
use crate::core::ics03_connection::msgs::{
    conn_open_ack, conn_open_confirm, conn_open_init, conn_open_try, ConnectionMsg,
};
use crate::core::ics04_channel::msgs::{
    acknowledgement, chan_close_confirm, chan_close_init, chan_open_ack, chan_open_confirm,
    chan_open_init, chan_open_try, recv_packet, timeout, timeout_on_close, ChannelMsg, PacketMsg,
};
use crate::Signer;

/// Trait to be implemented by all IBC messages
pub trait Msg: Clone {
    type Raw: From<Self> + prost::Message;

    /// Unique type identifier for this message, to support encoding to/from `prost_types::Any`.
    fn type_url(&self) -> String;

    fn signer(&self) -> Signer {
        Signer::new_empty()
    }

    fn get_sign_bytes(self) -> Vec<u8> {
        let raw_msg: Self::Raw = self.into();
        prost::Message::encode_to_vec(&raw_msg)
    }

    fn to_any(self) -> Any {
        Any {
            type_url: self.type_url(),
            value: self.get_sign_bytes(),
        }
    }
}

/// Enumeration of all messages that the local ICS26 module is capable of routing.
#[derive(Clone, Debug)]
pub enum MsgEnvelope {
    Client(ClientMsg),
    Connection(ConnectionMsg),
    Channel(ChannelMsg),
    Packet(PacketMsg),
}

impl MsgEnvelope {
    pub fn signer(&self) -> Signer {
        match self {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(msg) => msg.signer(),
                ClientMsg::UpdateClient(msg) => msg.signer(),
                ClientMsg::UpgradeClient(msg) => msg.signer(),
                ClientMsg::Misbehaviour(msg) => msg.signer(),
            },
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(msg) => msg.signer(),
                ConnectionMsg::OpenTry(msg) => msg.signer(),
                ConnectionMsg::OpenAck(msg) => msg.signer(),
                ConnectionMsg::OpenConfirm(msg) => msg.signer(),
            },
            MsgEnvelope::Channel(msg) => match msg {
                ChannelMsg::OpenInit(msg) => msg.signer(),
                ChannelMsg::OpenTry(msg) => msg.signer(),
                ChannelMsg::OpenAck(msg) => msg.signer(),
                ChannelMsg::OpenConfirm(msg) => msg.signer(),
                ChannelMsg::CloseInit(msg) => msg.signer(),
                ChannelMsg::CloseConfirm(msg) => msg.signer(),
            },
            MsgEnvelope::Packet(msg) => match msg {
                PacketMsg::Recv(msg) => msg.signer(),
                PacketMsg::Ack(msg) => msg.signer(),
                PacketMsg::Timeout(msg) => msg.signer(),
                PacketMsg::TimeoutOnClose(msg) => msg.signer(),
            },
        }
    }

    pub fn type_url(&self) -> String {
        match self {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(msg) => msg.type_url(),
                ClientMsg::UpdateClient(msg) => msg.type_url(),
                ClientMsg::UpgradeClient(msg) => msg.type_url(),
                ClientMsg::Misbehaviour(msg) => msg.type_url(),
            },
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(msg) => msg.type_url(),
                ConnectionMsg::OpenTry(msg) => msg.type_url(),
                ConnectionMsg::OpenAck(msg) => msg.type_url(),
                ConnectionMsg::OpenConfirm(msg) => msg.type_url(),
            },
            MsgEnvelope::Channel(msg) => match msg {
                ChannelMsg::OpenInit(msg) => msg.type_url(),
                ChannelMsg::OpenTry(msg) => msg.type_url(),
                ChannelMsg::OpenAck(msg) => msg.type_url(),
                ChannelMsg::OpenConfirm(msg) => msg.type_url(),
                ChannelMsg::CloseInit(msg) => msg.type_url(),
                ChannelMsg::CloseConfirm(msg) => msg.type_url(),
            },
            MsgEnvelope::Packet(msg) => match msg {
                PacketMsg::Recv(msg) => msg.type_url(),
                PacketMsg::Ack(msg) => msg.type_url(),
                PacketMsg::Timeout(msg) => msg.type_url(),
                PacketMsg::TimeoutOnClose(msg) => msg.type_url(),
            },
        }
    }
}

impl TryFrom<Any> for MsgEnvelope {
    type Error = RouterError;

    fn try_from(any_msg: Any) -> Result<Self, Self::Error> {
        match any_msg.type_url.as_str() {
            // ICS2 messages
            create_client::TYPE_URL => {
                // Pop out the message and then wrap it in the corresponding type.
                let domain_msg = create_client::MsgCreateClient::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Client(ClientMsg::CreateClient(domain_msg)))
            }
            update_client::TYPE_URL => {
                let domain_msg = update_client::MsgUpdateClient::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Client(ClientMsg::UpdateClient(domain_msg)))
            }
            upgrade_client::TYPE_URL => {
                let domain_msg = upgrade_client::MsgUpgradeClient::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Client(ClientMsg::UpgradeClient(domain_msg)))
            }
            misbehaviour::TYPE_URL => {
                let domain_msg = misbehaviour::MsgSubmitMisbehaviour::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Client(ClientMsg::Misbehaviour(domain_msg)))
            }

            // ICS03
            conn_open_init::TYPE_URL => {
                let domain_msg = conn_open_init::MsgConnectionOpenInit::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenInit(domain_msg)))
            }
            conn_open_try::TYPE_URL => {
                let domain_msg = conn_open_try::MsgConnectionOpenTry::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenTry(domain_msg)))
            }
            conn_open_ack::TYPE_URL => {
                let domain_msg = conn_open_ack::MsgConnectionOpenAck::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenAck(domain_msg)))
            }
            conn_open_confirm::TYPE_URL => {
                let domain_msg =
                    conn_open_confirm::MsgConnectionOpenConfirm::decode_vec(&any_msg.value)
                        .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenConfirm(
                    domain_msg,
                )))
            }

            // ICS04 channel messages
            chan_open_init::TYPE_URL => {
                let domain_msg = chan_open_init::MsgChannelOpenInit::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenInit(domain_msg)))
            }
            chan_open_try::TYPE_URL => {
                let domain_msg = chan_open_try::MsgChannelOpenTry::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenTry(domain_msg)))
            }
            chan_open_ack::TYPE_URL => {
                let domain_msg = chan_open_ack::MsgChannelOpenAck::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenAck(domain_msg)))
            }
            chan_open_confirm::TYPE_URL => {
                let domain_msg =
                    chan_open_confirm::MsgChannelOpenConfirm::decode_vec(&any_msg.value)
                        .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenConfirm(domain_msg)))
            }
            chan_close_init::TYPE_URL => {
                let domain_msg = chan_close_init::MsgChannelCloseInit::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::CloseInit(domain_msg)))
            }
            chan_close_confirm::TYPE_URL => {
                let domain_msg =
                    chan_close_confirm::MsgChannelCloseConfirm::decode_vec(&any_msg.value)
                        .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Channel(ChannelMsg::CloseConfirm(domain_msg)))
            }
            // ICS04 packet messages
            recv_packet::TYPE_URL => {
                let domain_msg = recv_packet::MsgRecvPacket::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Recv(domain_msg)))
            }
            acknowledgement::TYPE_URL => {
                let domain_msg = acknowledgement::MsgAcknowledgement::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Ack(domain_msg)))
            }
            timeout::TYPE_URL => {
                let domain_msg = timeout::MsgTimeout::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Timeout(domain_msg)))
            }
            timeout_on_close::TYPE_URL => {
                let domain_msg = timeout_on_close::MsgTimeoutOnClose::decode_vec(&any_msg.value)
                    .map_err(RouterError::MalformedMessageBytes)?;
                Ok(MsgEnvelope::Packet(PacketMsg::TimeoutOnClose(domain_msg)))
            }
            _ => Err(RouterError::UnknownMessageTypeUrl {
                url: any_msg.type_url,
            }),
        }
    }
}
