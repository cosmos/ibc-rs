//! Defines the `CosmosTx` message type, which sends a list of messages to hosts

use alloc::string::ToString;
use alloc::vec::Vec;

use ibc_core::primitives::proto::Protobuf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::apps::interchain_accounts::v1::CosmosTx as RawCosmosTx;

use crate::error::InterchainAccountError;

const TYPE_URL: &str = "/ibc.applications.interchain_accounts.v1.CosmosTx";

#[derive(Clone, Debug)]
pub struct CosmosTx {
    /// The list of messages to be executed on the host chain.
    pub messages: Vec<Any>,
}

impl Protobuf<RawCosmosTx> for CosmosTx {}

impl TryFrom<RawCosmosTx> for CosmosTx {
    type Error = InterchainAccountError;

    fn try_from(raw: RawCosmosTx) -> Result<Self, Self::Error> {
        if raw.messages.is_empty() {
            return Err(InterchainAccountError::empty("msgs of CosmosTx"));
        }

        Ok(CosmosTx {
            messages: raw.messages,
        })
    }
}

impl From<CosmosTx> for RawCosmosTx {
    fn from(value: CosmosTx) -> Self {
        RawCosmosTx {
            messages: value.messages,
        }
    }
}

impl From<CosmosTx> for Any {
    fn from(value: CosmosTx) -> Self {
        Any {
            type_url: TYPE_URL.to_string(),
            value: value.encode_vec(),
        }
    }
}
