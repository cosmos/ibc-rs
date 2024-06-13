use ibc_core_channel_types::msgs::{
    ChannelMsg, MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
    MsgTimeoutOnClose, PacketMsg, ACKNOWLEDGEMENT_TYPE_URL, CHAN_CLOSE_CONFIRM_TYPE_URL,
    CHAN_CLOSE_INIT_TYPE_URL, CHAN_OPEN_ACK_TYPE_URL, CHAN_OPEN_CONFIRM_TYPE_URL,
    CHAN_OPEN_INIT_TYPE_URL, CHAN_OPEN_TRY_TYPE_URL, RECV_PACKET_TYPE_URL,
    TIMEOUT_ON_CLOSE_TYPE_URL, TIMEOUT_TYPE_URL,
};
use ibc_core_client_types::msgs::RECOVER_CLIENT_TYPE_URL;
#[allow(deprecated)]
use ibc_core_client_types::msgs::{
    ClientMsg, MsgCreateClient, MsgSubmitMisbehaviour, MsgUpdateClient, MsgUpgradeClient,
    CREATE_CLIENT_TYPE_URL, SUBMIT_MISBEHAVIOUR_TYPE_URL, UPDATE_CLIENT_TYPE_URL,
    UPGRADE_CLIENT_TYPE_URL,
};
use ibc_core_connection_types::msgs::{
    ConnectionMsg, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, CONN_OPEN_ACK_TYPE_URL, CONN_OPEN_CONFIRM_TYPE_URL,
    CONN_OPEN_INIT_TYPE_URL, CONN_OPEN_TRY_TYPE_URL,
};
use ibc_core_router_types::error::RouterError;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::Protobuf;

/// Enumeration of all messages that the local ICS26 module is capable of routing.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum MsgEnvelope {
    Client(ClientMsg),
    Connection(ConnectionMsg),
    Channel(ChannelMsg),
    Packet(PacketMsg),
}

impl MsgEnvelope {
    pub fn signer(&self) -> Signer {
        match self.clone() {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(msg) => msg.signer,
                ClientMsg::UpdateClient(msg) => msg.signer,
                ClientMsg::UpgradeClient(msg) => msg.signer,
                ClientMsg::RecoverClient(msg) => msg.signer,
                #[allow(deprecated)]
                ClientMsg::Misbehaviour(msg) => msg.signer,
            },
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(msg) => msg.signer,
                ConnectionMsg::OpenTry(msg) => msg.signer,
                ConnectionMsg::OpenAck(msg) => msg.signer,
                ConnectionMsg::OpenConfirm(msg) => msg.signer,
            },
            MsgEnvelope::Channel(msg) => match msg {
                ChannelMsg::OpenInit(msg) => msg.signer,
                ChannelMsg::OpenTry(msg) => msg.signer,
                ChannelMsg::OpenAck(msg) => msg.signer,
                ChannelMsg::OpenConfirm(msg) => msg.signer,
                ChannelMsg::CloseInit(msg) => msg.signer,
                ChannelMsg::CloseConfirm(msg) => msg.signer,
            },
            MsgEnvelope::Packet(msg) => match msg {
                PacketMsg::Recv(msg) => msg.signer,
                PacketMsg::Ack(msg) => msg.signer,
                PacketMsg::Timeout(msg) => msg.signer,
                PacketMsg::TimeoutOnClose(msg) => msg.signer,
            },
        }
    }

    pub fn type_url(&self) -> String {
        match self {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(_msg) => CREATE_CLIENT_TYPE_URL.into(),
                ClientMsg::UpdateClient(_msg) => UPDATE_CLIENT_TYPE_URL.into(),
                ClientMsg::UpgradeClient(_msg) => UPGRADE_CLIENT_TYPE_URL.into(),
                ClientMsg::Misbehaviour(_msg) => SUBMIT_MISBEHAVIOUR_TYPE_URL.into(),
                ClientMsg::RecoverClient(_msg) => RECOVER_CLIENT_TYPE_URL.into(),
            },
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(_msg) => CONN_OPEN_INIT_TYPE_URL.into(),
                ConnectionMsg::OpenTry(_msg) => CONN_OPEN_TRY_TYPE_URL.into(),
                ConnectionMsg::OpenAck(_msg) => CONN_OPEN_ACK_TYPE_URL.into(),
                ConnectionMsg::OpenConfirm(_msg) => CONN_OPEN_CONFIRM_TYPE_URL.into(),
            },
            MsgEnvelope::Channel(msg) => match msg {
                ChannelMsg::OpenInit(_msg) => CHAN_OPEN_INIT_TYPE_URL.into(),
                ChannelMsg::OpenTry(_msg) => CHAN_OPEN_TRY_TYPE_URL.into(),
                ChannelMsg::OpenAck(_msg) => CHAN_OPEN_ACK_TYPE_URL.into(),
                ChannelMsg::OpenConfirm(_msg) => CHAN_OPEN_CONFIRM_TYPE_URL.into(),
                ChannelMsg::CloseInit(_msg) => CHAN_CLOSE_INIT_TYPE_URL.into(),
                ChannelMsg::CloseConfirm(_msg) => CHAN_CLOSE_CONFIRM_TYPE_URL.into(),
            },
            MsgEnvelope::Packet(msg) => match msg {
                PacketMsg::Recv(_msg) => RECV_PACKET_TYPE_URL.into(),
                PacketMsg::Ack(_msg) => ACKNOWLEDGEMENT_TYPE_URL.into(),
                PacketMsg::Timeout(_msg) => TIMEOUT_TYPE_URL.into(),
                PacketMsg::TimeoutOnClose(_msg) => TIMEOUT_ON_CLOSE_TYPE_URL.into(),
            },
        }
    }
}

#[allow(deprecated)]
impl TryFrom<Any> for MsgEnvelope {
    type Error = RouterError;

    fn try_from(any_msg: Any) -> Result<Self, Self::Error> {
        match any_msg.type_url.as_str() {
            // ICS2 messages
            CREATE_CLIENT_TYPE_URL => {
                // Pop out the message and then wrap it in the corresponding type.
                let domain_msg = MsgCreateClient::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Client(ClientMsg::CreateClient(domain_msg)))
            }
            UPDATE_CLIENT_TYPE_URL => {
                let domain_msg = MsgUpdateClient::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Client(ClientMsg::UpdateClient(domain_msg)))
            }
            UPGRADE_CLIENT_TYPE_URL => {
                let domain_msg = MsgUpgradeClient::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Client(ClientMsg::UpgradeClient(domain_msg)))
            }
            SUBMIT_MISBEHAVIOUR_TYPE_URL => {
                let domain_msg =
                    MsgSubmitMisbehaviour::decode_vec(&any_msg.value).map_err(|e| {
                        RouterError::MalformedMessageBytes {
                            reason: e.to_string(),
                        }
                    })?;
                Ok(MsgEnvelope::Client(ClientMsg::Misbehaviour(domain_msg)))
            }

            // ICS03
            CONN_OPEN_INIT_TYPE_URL => {
                let domain_msg =
                    MsgConnectionOpenInit::decode_vec(&any_msg.value).map_err(|e| {
                        RouterError::MalformedMessageBytes {
                            reason: e.to_string(),
                        }
                    })?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenInit(domain_msg)))
            }
            CONN_OPEN_TRY_TYPE_URL => {
                let domain_msg = MsgConnectionOpenTry::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenTry(domain_msg)))
            }
            CONN_OPEN_ACK_TYPE_URL => {
                let domain_msg = MsgConnectionOpenAck::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenAck(domain_msg)))
            }
            CONN_OPEN_CONFIRM_TYPE_URL => {
                let domain_msg =
                    MsgConnectionOpenConfirm::decode_vec(&any_msg.value).map_err(|e| {
                        RouterError::MalformedMessageBytes {
                            reason: e.to_string(),
                        }
                    })?;
                Ok(MsgEnvelope::Connection(ConnectionMsg::OpenConfirm(
                    domain_msg,
                )))
            }

            // ICS04 channel messages
            CHAN_OPEN_INIT_TYPE_URL => {
                let domain_msg = MsgChannelOpenInit::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenInit(domain_msg)))
            }
            CHAN_OPEN_TRY_TYPE_URL => {
                let domain_msg = MsgChannelOpenTry::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenTry(domain_msg)))
            }
            CHAN_OPEN_ACK_TYPE_URL => {
                let domain_msg = MsgChannelOpenAck::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenAck(domain_msg)))
            }
            CHAN_OPEN_CONFIRM_TYPE_URL => {
                let domain_msg =
                    MsgChannelOpenConfirm::decode_vec(&any_msg.value).map_err(|e| {
                        RouterError::MalformedMessageBytes {
                            reason: e.to_string(),
                        }
                    })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::OpenConfirm(domain_msg)))
            }
            CHAN_CLOSE_INIT_TYPE_URL => {
                let domain_msg = MsgChannelCloseInit::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::CloseInit(domain_msg)))
            }
            CHAN_CLOSE_CONFIRM_TYPE_URL => {
                let domain_msg =
                    MsgChannelCloseConfirm::decode_vec(&any_msg.value).map_err(|e| {
                        RouterError::MalformedMessageBytes {
                            reason: e.to_string(),
                        }
                    })?;
                Ok(MsgEnvelope::Channel(ChannelMsg::CloseConfirm(domain_msg)))
            }
            // ICS04 packet messages
            RECV_PACKET_TYPE_URL => {
                let domain_msg = MsgRecvPacket::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Packet(PacketMsg::Recv(domain_msg)))
            }
            ACKNOWLEDGEMENT_TYPE_URL => {
                let domain_msg = MsgAcknowledgement::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Packet(PacketMsg::Ack(domain_msg)))
            }
            TIMEOUT_TYPE_URL => {
                let domain_msg = MsgTimeout::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Packet(PacketMsg::Timeout(domain_msg)))
            }
            TIMEOUT_ON_CLOSE_TYPE_URL => {
                let domain_msg = MsgTimeoutOnClose::decode_vec(&any_msg.value).map_err(|e| {
                    RouterError::MalformedMessageBytes {
                        reason: e.to_string(),
                    }
                })?;
                Ok(MsgEnvelope::Packet(PacketMsg::TimeoutOnClose(domain_msg)))
            }
            _ => Err(RouterError::UnknownMessageTypeUrl {
                url: any_msg.type_url,
            }),
        }
    }
}
