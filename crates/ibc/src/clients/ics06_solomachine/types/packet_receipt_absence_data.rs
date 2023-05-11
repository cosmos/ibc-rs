use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::PacketReceiptAbsenceData as RawPacketReceiptAbsenceData;
use ibc_proto::protobuf::Protobuf;

/// PacketReceiptAbsenceData returns the SignBytes data for
/// packet receipt absence verification.
#[derive(Clone, PartialEq)]
pub struct PacketReceiptAbsenceData {
    pub path: Vec<u8>,
}

impl Protobuf<RawPacketReceiptAbsenceData> for PacketReceiptAbsenceData {}

impl TryFrom<RawPacketReceiptAbsenceData> for PacketReceiptAbsenceData {
    type Error = Error;

    fn try_from(raw: RawPacketReceiptAbsenceData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<PacketReceiptAbsenceData> for RawPacketReceiptAbsenceData {
    fn from(value: PacketReceiptAbsenceData) -> Self {
        todo!()
    }
}
