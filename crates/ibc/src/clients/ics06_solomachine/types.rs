use core::fmt::write;

use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::DataType as RawDataType;

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

impl core::fmt::Display for DataType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            DataType::UninitializedUnspecified => write!(f, "uninitialized unspecified"),
            DataType::ClientState => write!(f, "client state"),
            DataType::ConsensusState => write!(f, "consensus state"),
            DataType::ConnectionState => write!(f, "connection state"),
            DataType::ChannelState => write!(f, "channel state"),
            DataType::PacketCommitment => write!(f, "packet commitment"),
            DataType::PacketAcknowledgement => write!(f, "packet acknowledgement"),
            DataType::PacketReceiptAbsence => write!(f, "packet receipt absence"),
            DataType::NextSequenceRecv => write!(f, "next sequence recv"),
            DataType::Header => write!(f, "header"),
        }
    }
}

impl From<RawDataType> for DataType {
    fn from(raw: RawDataType) -> Self {
        match raw {
            RawDataType::UninitializedUnspecified => Self::UninitializedUnspecified,
            RawDataType::ClientState => Self::ClientState,
            RawDataType::ConsensusState => Self::ConsensusState,
            RawDataType::ConnectionState => Self::ConnectionState,
            RawDataType::ChannelState => Self::ChannelState,
            RawDataType::PacketCommitment => Self::PacketCommitment,
            RawDataType::PacketAcknowledgement => Self::PacketAcknowledgement,
            RawDataType::PacketReceiptAbsence => Self::PacketReceiptAbsence,
            RawDataType::NextSequenceRecv => Self::NextSequenceRecv,
            RawDataType::Header => Self::Header,
        }
    }
}

impl From<DataType> for RawDataType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::UninitializedUnspecified => Self::UninitializedUnspecified,
            DataType::ClientState => Self::ClientState,
            DataType::ConsensusState => Self::ConsensusState,
            DataType::ConnectionState => Self::ConnectionState,
            DataType::ChannelState => Self::ChannelState,
            DataType::PacketCommitment => Self::PacketCommitment,
            DataType::PacketAcknowledgement => Self::PacketAcknowledgement,
            DataType::PacketReceiptAbsence => Self::PacketReceiptAbsence,
            DataType::NextSequenceRecv => Self::NextSequenceRecv,
            DataType::Header => Self::Header,
        }
    }
}

impl TryFrom<i32> for DataType {
    type Error = Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let data_type = match value {
            0 => Self::UninitializedUnspecified,
            1 => Self::ClientState,
            2 => Self::ConsensusState,
            3 => Self::ConnectionState,
            4 => Self::ChannelState,
            5 => Self::PacketCommitment,
            6 => Self::PacketAcknowledgement,
            7 => Self::PacketReceiptAbsence,
            8 => Self::NextSequenceRecv,
            9 => Self::Header,
            i => return Err(Error::UnknownDataType(i)),
        };
        Ok(data_type)
    }
}

impl From<DataType> for i32 {
    fn from(value: DataType) -> Self {
        match value {
            DataType::UninitializedUnspecified => 0,
            DataType::ClientState => 1,
            DataType::ConsensusState => 2,
            DataType::ConnectionState => 3,
            DataType::ChannelState => 4,
            DataType::PacketCommitment => 5,
            DataType::PacketAcknowledgement => 6,
            DataType::PacketReceiptAbsence => 7,
            DataType::NextSequenceRecv => 8,
            DataType::Header => 9,
        }
    }
}
