use alloc::string::{String, ToString};

pub use ibc_proto::ibc::applications::interchain_accounts::controller::v1::MsgSendTx as RawMsgSendTx;
use ibc_proto::protobuf::Protobuf;

use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::applications::interchain_accounts::packet::InterchainAccountPacketData;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::timestamp::Timestamp;
use crate::core::Msg;
use crate::Signer;

pub(crate) const TYPE_URL: &str = "/ibc.applications.interchain_account.controller.v1.MsgSendTx";

/// Defines the domain type for the `MsgSendTx` message.
#[derive(Clone, Debug)]
pub struct MsgSendTx {
    /// The controller owner address
    pub owner: Signer,
    /// The connection id on the controller chain
    pub conn_id_on_a: ConnectionId,
    /// The packet data
    pub packet_data: InterchainAccountPacketData,
    /// The relative timeout
    pub relative_timeout: Timestamp,
}

impl Msg for MsgSendTx {
    type Raw = RawMsgSendTx;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgSendTx> for MsgSendTx {}

impl TryFrom<RawMsgSendTx> for MsgSendTx {
    type Error = InterchainAccountError;

    fn try_from(raw: RawMsgSendTx) -> Result<Self, Self::Error> {
        let relative_timeout = Timestamp::from_nanoseconds(raw.relative_timeout)
            .map_err(InterchainAccountError::source)?;

        if !relative_timeout.is_set() {
            return Err(InterchainAccountError::empty("relative timeout is not set"));
        }

        Ok(MsgSendTx {
            owner: raw.owner.into(),
            conn_id_on_a: raw
                .connection_id
                .parse()
                .map_err(InterchainAccountError::source)?,
            packet_data: match raw.packet_data {
                Some(packet_data) => packet_data.try_into()?,
                None => Err(InterchainAccountError::empty("packet data"))?,
            },
            relative_timeout,
        })
    }
}

impl From<MsgSendTx> for RawMsgSendTx {
    fn from(domain: MsgSendTx) -> Self {
        RawMsgSendTx {
            owner: domain.owner.to_string(),
            connection_id: domain.conn_id_on_a.to_string(),
            packet_data: Some(domain.packet_data.into()),
            relative_timeout: domain.relative_timeout.nanoseconds(),
        }
    }
}
