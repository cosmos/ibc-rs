use ibc_eureka_core_channel_types::msgs::{
    MsgAcknowledgement, MsgRecvPacket, MsgTimeout, PacketMsg, ACKNOWLEDGEMENT_TYPE_URL,
    RECV_PACKET_TYPE_URL, TIMEOUT_TYPE_URL,
};
#[allow(deprecated)]
use ibc_eureka_core_client_types::msgs::{
    ClientMsg, MsgCreateClient, MsgSubmitMisbehaviour, MsgUpdateClient, MsgUpgradeClient,
    CREATE_CLIENT_TYPE_URL, SUBMIT_MISBEHAVIOUR_TYPE_URL, UPDATE_CLIENT_TYPE_URL,
    UPGRADE_CLIENT_TYPE_URL,
};
use ibc_eureka_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
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
    // Channel(ChannelMsg),
    Packet(PacketMsg),
}

#[allow(deprecated)]
impl TryFrom<Any> for MsgEnvelope {
    type Error = DecodingError;

    fn try_from(any_msg: Any) -> Result<Self, Self::Error> {
        match any_msg.type_url.as_str() {
            // ICS2 messages
            CREATE_CLIENT_TYPE_URL => {
                // Pop out the message and then wrap it in the corresponding type.
                let domain_msg = MsgCreateClient::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Client(ClientMsg::CreateClient(domain_msg)))
            }
            UPDATE_CLIENT_TYPE_URL => {
                let domain_msg = MsgUpdateClient::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Client(ClientMsg::UpdateClient(domain_msg)))
            }
            UPGRADE_CLIENT_TYPE_URL => {
                let domain_msg = MsgUpgradeClient::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Client(ClientMsg::UpgradeClient(domain_msg)))
            }
            SUBMIT_MISBEHAVIOUR_TYPE_URL => {
                let domain_msg = MsgSubmitMisbehaviour::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Client(ClientMsg::Misbehaviour(domain_msg)))
            }

            // ICS04 packet messages
            RECV_PACKET_TYPE_URL => {
                let domain_msg = MsgRecvPacket::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Recv(domain_msg)))
            }
            ACKNOWLEDGEMENT_TYPE_URL => {
                let domain_msg = MsgAcknowledgement::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Ack(domain_msg)))
            }
            TIMEOUT_TYPE_URL => {
                let domain_msg = MsgTimeout::decode_vec(&any_msg.value)?;
                Ok(MsgEnvelope::Packet(PacketMsg::Timeout(domain_msg)))
            }
            _ => Err(DecodingError::UnknownTypeUrl(any_msg.type_url))?,
        }
    }
}
