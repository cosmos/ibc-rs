//! Defines the Non-Fungible Token Transfer message type

use ibc_core::channel::types::error::PacketError;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Timestamp;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::applications::nft_transfer::v1::MsgTransfer as RawMsgTransfer;
use ibc_proto::Protobuf;

use crate::error::NftTransferError;
use crate::packet::PacketData;

pub(crate) const TYPE_URL: &str = "/ibc.applications.nft_transfer.v1.MsgTransfer";

/// Message used to build an ICS-721 Non-Fungible Token Transfer packet.
///
/// Note that this message is not a packet yet, as it lacks the proper sequence
/// number, and destination port/channel. This is by design. The sender of the
/// packet, which might be the user of a command line application, should only
/// have to specify the information related to the transfer of the token, and
/// let the library figure out how to build the packet properly.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode,)
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MsgTransfer {
    /// the port on which the packet will be sent
    pub port_id_on_a: PortId,
    /// the channel by which the packet will be sent
    pub chan_id_on_a: ChannelId,
    /// NFT transfer packet data of the packet that will be sent
    pub packet_data: PacketData,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to None.
    pub timeout_height_on_b: TimeoutHeight,
    /// Timeout timestamp relative to the current block timestamp.
    /// The timeout is disabled when set to 0.
    pub timeout_timestamp_on_b: Timestamp,
}

impl TryFrom<RawMsgTransfer> for MsgTransfer {
    type Error = NftTransferError;

    fn try_from(raw_msg: RawMsgTransfer) -> Result<Self, Self::Error> {
        let timeout_timestamp_on_b = Timestamp::from_nanoseconds(raw_msg.timeout_timestamp)
            .map_err(PacketError::InvalidPacketTimestamp)
            .map_err(ContextError::from)?;

        let timeout_height_on_b: TimeoutHeight = raw_msg
            .timeout_height
            .try_into()
            .map_err(ContextError::from)?;

        // Packet timeout height and packet timeout timestamp cannot both be unset.
        if !timeout_height_on_b.is_set() && !timeout_timestamp_on_b.is_set() {
            return Err(ContextError::from(PacketError::MissingTimeout))?;
        }

        let memo = if raw_msg.memo.is_empty() {
            None
        } else {
            Some(raw_msg.memo.into())
        };

        Ok(MsgTransfer {
            port_id_on_a: raw_msg.source_port.parse()?,
            chan_id_on_a: raw_msg.source_channel.parse()?,
            packet_data: PacketData {
                class_id: raw_msg.class_id.parse()?,
                class_uri: None,
                class_data: None,
                token_ids: raw_msg.token_ids.try_into()?,
                token_uris: None,
                token_data: None,
                sender: raw_msg.sender.into(),
                receiver: raw_msg.receiver.into(),
                memo,
            },
            timeout_height_on_b,
            timeout_timestamp_on_b,
        })
    }
}

impl From<MsgTransfer> for RawMsgTransfer {
    fn from(domain_msg: MsgTransfer) -> Self {
        RawMsgTransfer {
            source_port: domain_msg.port_id_on_a.to_string(),
            source_channel: domain_msg.chan_id_on_a.to_string(),
            class_id: domain_msg.packet_data.class_id.to_string(),
            token_ids: domain_msg
                .packet_data
                .token_ids
                .as_ref()
                .iter()
                .map(|t| t.to_string())
                .collect(),
            sender: domain_msg.packet_data.sender.to_string(),
            receiver: domain_msg.packet_data.receiver.to_string(),
            timeout_height: domain_msg.timeout_height_on_b.into(),
            timeout_timestamp: domain_msg.timeout_timestamp_on_b.nanoseconds(),
            memo: domain_msg
                .packet_data
                .memo
                .map(|m| m.to_string())
                .unwrap_or_default(),
        }
    }
}

impl Protobuf<RawMsgTransfer> for MsgTransfer {}

impl TryFrom<Any> for MsgTransfer {
    type Error = NftTransferError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            TYPE_URL => {
                MsgTransfer::decode_vec(&raw.value).map_err(|e| NftTransferError::DecodeRawMsg {
                    reason: e.to_string(),
                })
            }
            _ => Err(NftTransferError::UnknownMsgType {
                msg_type: raw.type_url,
            }),
        }
    }
}
