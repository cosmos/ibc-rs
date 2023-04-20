use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::DataType as RawDataType;

/// DataType defines the type of solo machine proof being created. This is done
/// to preserve uniqueness of different data sign byte encodings.
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
