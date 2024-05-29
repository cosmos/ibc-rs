use alloc::string::String;
use alloc::vec::Vec;
use ibc_proto::ibc::applications::interchain_accounts::host::v1::Params as RawParams;
use ibc_proto::protobuf::Protobuf;

use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::core::MsgEnvelope;

pub const ALLOW_ALL_HOST_MSGS: &str = "*";

/// Defines the interchain account host parameters.
#[derive(Clone, Debug)]
pub struct Params {
    /// Enables or disables the host submodule.
    pub host_enabled: bool,
    /// Defines a list of message typeURLs allowed to be executed on a host chain.
    pub allow_messages: Vec<String>,
}

impl Params {
    pub fn new(host_enabled: bool, allow_messages: Vec<String>) -> Self {
        Params {
            host_enabled,
            allow_messages,
        }
    }

    pub fn contains_msg_type(&self, msg: &MsgEnvelope) -> bool {
        if self.allow_messages.len() == 1 && self.allow_messages[0] == ALLOW_ALL_HOST_MSGS {
            true
        } else {
            self.allow_messages.contains(&msg.type_url())
        }
    }
}

impl Protobuf<RawParams> for Params {}

impl TryFrom<RawParams> for Params {
    type Error = InterchainAccountError;

    fn try_from(raw: RawParams) -> Result<Self, Self::Error> {
        if raw.allow_messages.is_empty() {
            return Err(InterchainAccountError::empty("allow_messages"));
        }

        if raw.allow_messages.iter().any(|m| m.trim().is_empty()) {
            return Err(InterchainAccountError::empty(
                "allow_messages cannot contain empty strings",
            ));
        }

        Ok(Params {
            host_enabled: raw.host_enabled,
            allow_messages: raw.allow_messages,
        })
    }
}

impl From<Params> for RawParams {
    fn from(value: Params) -> Self {
        RawParams {
            host_enabled: value.host_enabled,
            allow_messages: value.allow_messages,
        }
    }
}
