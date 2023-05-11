use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::PacketCommitmentData as RawPacketCommitmentData;
use ibc_proto::protobuf::Protobuf;

/// PacketCommitmentData returns the SignBytes data for packet commitment
/// verification.
#[derive(Clone, PartialEq)]
pub struct PacketCommitmentData {
    pub path: Vec<u8>,
    pub commitment: Vec<u8>,
}

impl Protobuf<RawPacketCommitmentData> for PacketCommitmentData {}

impl TryFrom<RawPacketCommitmentData> for PacketCommitmentData {
    type Error = Error;

    fn try_from(raw: RawPacketCommitmentData) -> Result<Self, Self::Error> {
        Ok(Self {
            path: raw.path,
            commitment: raw.commitment,
        })
    }
}

impl From<PacketCommitmentData> for RawPacketCommitmentData {
    fn from(value: PacketCommitmentData) -> Self {
        Self {
            path: value.path,
            commitment: value.commitment,
        }
    }
}
