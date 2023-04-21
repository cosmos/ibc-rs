use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::DataType as RawDataType;

pub mod channel_state_data;
pub mod client_stata_data;
pub mod connection_state_data;
pub mod consensus_state_data;
pub mod header_data;
pub mod next_sequence_recv_data;
pub mod packet_acknowledgement_data;
pub mod packet_commitment_data;
pub mod packet_receipt_absence_data;
pub mod sign_bytes;
pub mod timestamped_signature_data;

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
