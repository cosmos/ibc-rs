//! Definition of domain type message `MsgUpdateClient`.

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
use ibc_proto::Protobuf;

use crate::error::ClientError;

pub const UPDATE_CLIENT_TYPE_URL: &str = "/ibc.core.client.v1.MsgUpdateClient";

/// Represents the message that triggers the update of an on-chain (IBC) client
/// either with new headers, or evidence of misbehaviour.
/// Note that some types of misbehaviour can be detected when a headers
/// are updated (`UpdateKind::UpdateClient`).
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Any,
    pub signer: Signer,
}

impl Protobuf<RawMsgUpdateClient> for MsgUpdateClient {}

impl TryFrom<RawMsgUpdateClient> for MsgUpdateClient {
    type Error = ClientError;

    fn try_from(raw: RawMsgUpdateClient) -> Result<Self, Self::Error> {
        Ok(MsgUpdateClient {
            client_id: raw
                .client_id
                .parse()
                .map_err(ClientError::InvalidMsgUpdateClientId)?,
            client_message: raw
                .client_message
                .ok_or(ClientError::MissingClientMessage)?,
            signer: raw.signer.into(),
        })
    }
}

impl From<MsgUpdateClient> for RawMsgUpdateClient {
    fn from(ics_msg: MsgUpdateClient) -> Self {
        RawMsgUpdateClient {
            client_id: ics_msg.client_id.to_string(),
            client_message: Some(ics_msg.client_message),
            signer: ics_msg.signer.to_string(),
        }
    }
}
