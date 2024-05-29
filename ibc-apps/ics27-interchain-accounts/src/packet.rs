use alloc::string::String;
use alloc::vec::Vec;

use crate::applications::interchain_accounts::error::InterchainAccountError;
pub use ibc_proto::ibc::applications::interchain_accounts::v1::InterchainAccountPacketData as RawInterchainAccountPacketData;
use ibc_proto::protobuf::Protobuf;

const MAX_MEMO_CHAR_LENGTH: usize = 256;

/// Defines the domain type for the interchain account packet data.
#[derive(Clone, Debug)]
pub struct InterchainAccountPacketData {
    /// The type of the packet data.
    pub packet_type: ICAPacketType,
    /// The data to be sent to the interchain accounts host chain.
    pub data: Vec<u8>,
    /// The memo to be included in the transaction.
    pub memo: String,
}

impl TryFrom<Vec<u8>> for InterchainAccountPacketData {
    type Error = InterchainAccountError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(InterchainAccountError::empty("packet data"));
        }
        Ok(Self {
            packet_type: ICAPacketType::ExecuteTx,
            data,
            memo: String::new(),
        })
    }
}

impl Protobuf<RawInterchainAccountPacketData> for InterchainAccountPacketData {}

impl TryFrom<RawInterchainAccountPacketData> for InterchainAccountPacketData {
    type Error = InterchainAccountError;

    fn try_from(raw: RawInterchainAccountPacketData) -> Result<Self, Self::Error> {
        let packet_type = match raw.r#type {
            0 => ICAPacketType::ExecuteTx,
            _ => {
                return Err(InterchainAccountError::invalid(
                    "packet data type must be of type ExecuteTx",
                ))
            }
        };

        if raw.data.is_empty() {
            return Err(InterchainAccountError::empty("packet data"));
        }

        if raw.memo.len() > MAX_MEMO_CHAR_LENGTH {
            return Err(InterchainAccountError::invalid(
                "packet memo cannot be greater than 256 characters",
            ));
        }

        Ok(InterchainAccountPacketData {
            packet_type,
            data: raw.data,
            memo: raw.memo,
        })
    }
}

impl From<InterchainAccountPacketData> for RawInterchainAccountPacketData {
    fn from(domain: InterchainAccountPacketData) -> Self {
        RawInterchainAccountPacketData {
            r#type: domain.packet_type as i32,
            data: domain.data,
            memo: domain.memo,
        }
    }
}

/// Defines a classification of message issued from a controller chain to its
/// associated interchain accounts host
#[derive(Clone, Debug)]
pub enum ICAPacketType {
    /// Execute a transaction on an interchain accounts host chain
    ExecuteTx = 0,
}
