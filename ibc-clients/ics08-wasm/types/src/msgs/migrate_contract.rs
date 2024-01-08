use core::str::FromStr;

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::lightclients::wasm::v1::MsgMigrateContract as RawMsgMigrateContract;
use ibc_proto::Protobuf;

use crate::error::Error;
use crate::Bytes;

pub const MIGRATE_CONTRACT_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.MsgMigrateContract";

/// Defines the message type for migrating a Wasm contract on the chain.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MsgMigrateContract {
    pub signer: Signer,
    pub client_id: ClientId,
    pub checksum: Bytes,
    pub msg: Bytes,
}

impl Protobuf<RawMsgMigrateContract> for MsgMigrateContract {}

impl From<MsgMigrateContract> for RawMsgMigrateContract {
    fn from(value: MsgMigrateContract) -> Self {
        Self {
            signer: value.signer.to_string(),
            client_id: value.client_id.to_string(),
            checksum: value.checksum,
            msg: value.msg,
        }
    }
}

impl TryFrom<RawMsgMigrateContract> for MsgMigrateContract {
    type Error = Error;

    fn try_from(value: RawMsgMigrateContract) -> Result<Self, Self::Error> {
        Ok(Self {
            signer: Signer::from(value.signer),
            client_id: ClientId::from_str(&value.client_id)?,
            checksum: value.checksum,
            msg: value.msg,
        })
    }
}
