use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::DataType as RawDataType;
use ibc_proto::ibc::lightclients::solomachine::v1::SignBytes as RawSignBytes;
use ibc_proto::ibc::lightclients::solomachine::v1::TimestampedSignatureData as RawTimestampedSignatureData;
use ibc_proto::protobuf::Protobuf;

/// DataType defines the type of solo machine proof being created. This is done
/// to preserve uniqueness of different data sign byte encodings.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataType {
    /// Default State
    UninitializedUnspecified,
    /// Data type for client state verification
    ClientState,
    /// Data type for consensus state verification
    ConsensusState,
    /// Data type for connection state verification
    ConnectionState,
    /// Data type for channel state verification
    ChannelState,
    /// Data type for packet commitment verification
    PacketCommitment,
    /// Data type for packet acknowledgement verification
    PacketAcknowledgement,
    /// Data type for packet receipt absence verification
    PacketReceiptAbsence,
    /// Data type for next sequence recv verification
    NextSequenceRecv,
    /// Data type for header verification
    Header,
}

impl TryFrom<RawDataType> for DataType {
    type Error = Error;

    fn try_from(raw: RawDataType) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<DataType> for RawDataType {
    fn from(value: DataType) -> Self {
        todo!()
    }
}

/// TimestampedSignatureData contains the signature data and the timestamp of the
/// signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct TimestampedSignatureData {
    pub signature_data: Vec<u8>,
    pub timestamp: u64,
}

impl Protobuf<RawTimestampedSignatureData> for TimestampedSignatureData {}

impl TryFrom<RawTimestampedSignatureData> for TimestampedSignatureData {
    type Error = Error;

    fn try_from(raw: RawTimestampedSignatureData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<TimestampedSignatureData> for RawTimestampedSignatureData {
    fn from(value: TimestampedSignatureData) -> Self {
        todo!()
    }
}

/// SignBytes defines the signed bytes used for signature verification.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct SignBytes {
    pub sequence: u64,
    pub timestamp: u64,
    pub diversifier: String,
    /// type of the data used
    pub data_type: i32,
    /// marshaled data
    pub data: Vec<u8>,
}

// impl Protobuf<RawSignBytes> for SignBytes {}

impl TryFrom<RawSignBytes> for SignBytes {
    type Error = Error;

    fn try_from(raw: RawSignBytes) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<SignBytes> for RawTimestampedSignatureData {
    fn from(value: SignBytes) -> Self {
        todo!()
    }
}
