use alloc::string::{String, ToString};
pub use ibc_proto::ibc::applications::interchain_accounts::controller::v1::MsgRegisterInterchainAccount as RawMsgRegisterInterchainAccount;
use ibc_proto::protobuf::Protobuf;

use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::Msg;
use crate::Signer;

pub(crate) const TYPE_URL: &str =
    "/ibc.applications.interchain_account.controller.v1.MsgRegisterInterchainAccount";

// Defines the domain type for the interchain account registration message.
#[derive(Clone, Debug)]
pub struct MsgRegisterInterchainAccount {
    /// The owner of the interchain account.
    pub owner: Signer,
    /// The connection identifier on the controller chain.
    /// Note: to learn about our naming convention, see [here](crate::applications::interchain_accounts).
    pub conn_id_on_a: ConnectionId,
    /// The version of the interchain account.
    pub version: Version,
}

impl MsgRegisterInterchainAccount {
    pub fn new(
        owner: Signer,
        conn_id_on_a: ConnectionId,
        version: Version,
    ) -> MsgRegisterInterchainAccount {
        MsgRegisterInterchainAccount {
            owner,
            conn_id_on_a,
            version,
        }
    }
}

impl Msg for MsgRegisterInterchainAccount {
    type Raw = RawMsgRegisterInterchainAccount;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgRegisterInterchainAccount> for MsgRegisterInterchainAccount {}

impl TryFrom<RawMsgRegisterInterchainAccount> for MsgRegisterInterchainAccount {
    type Error = InterchainAccountError;

    fn try_from(raw: RawMsgRegisterInterchainAccount) -> Result<Self, Self::Error> {
        if raw.owner.is_empty() {
            return Err(InterchainAccountError::empty("controller owner address"));
        }

        Ok(MsgRegisterInterchainAccount {
            owner: raw.owner.into(),
            conn_id_on_a: raw
                .connection_id
                .parse()
                .map_err(InterchainAccountError::source)?,
            version: raw
                .version
                .parse()
                .map_err(InterchainAccountError::source)?,
        })
    }
}

impl From<MsgRegisterInterchainAccount> for RawMsgRegisterInterchainAccount {
    fn from(domain: MsgRegisterInterchainAccount) -> Self {
        RawMsgRegisterInterchainAccount {
            owner: domain.owner.to_string(),
            connection_id: domain.conn_id_on_a.to_string(),
            version: domain.version.to_string(),
        }
    }
}
