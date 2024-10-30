//! Definition of domain type message `MsgCreateClient`.

use ibc_eureka_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgCreateClient as RawMsgCreateClient;
use ibc_proto::Protobuf;

pub const CREATE_CLIENT_TYPE_URL: &str = "/ibc.core.client.v1.MsgCreateClient";

/// A type of message that triggers the creation of a new on-chain (IBC) client.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgCreateClient {
    pub client_state: Any,
    pub consensus_state: Any,
    pub signer: Signer,
}

impl MsgCreateClient {
    pub fn new(client_state: Any, consensus_state: Any, signer: Signer) -> Self {
        MsgCreateClient {
            client_state,
            consensus_state,
            signer,
        }
    }
}

impl Protobuf<RawMsgCreateClient> for MsgCreateClient {}

impl TryFrom<RawMsgCreateClient> for MsgCreateClient {
    type Error = DecodingError;

    fn try_from(raw: RawMsgCreateClient) -> Result<Self, Self::Error> {
        let raw_client_state = raw
            .client_state
            .ok_or(DecodingError::missing_raw_data("client state"))?;

        let raw_consensus_state = raw
            .consensus_state
            .ok_or(DecodingError::missing_raw_data("consensus state"))?;

        Ok(MsgCreateClient::new(
            raw_client_state,
            raw_consensus_state,
            raw.signer.into(),
        ))
    }
}

impl From<MsgCreateClient> for RawMsgCreateClient {
    fn from(ics_msg: MsgCreateClient) -> Self {
        RawMsgCreateClient {
            client_state: Some(ics_msg.client_state),
            consensus_state: Some(ics_msg.consensus_state),
            signer: ics_msg.signer.to_string(),
        }
    }
}
