//! Definition of domain type message `MsgSubmitMisbehaviour`.

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any as ProtoAny;
use ibc_proto::ibc::core::client::v1::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;
use ibc_proto::Protobuf;

use crate::error::ClientError;

pub const SUBMIT_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.core.client.v1.MsgSubmitMisbehaviour";

/// A type of message that submits client misbehaviour proof.
///
/// Deprecated since v0.51.0. Misbehaviour reports should be submitted via the `MsgUpdateClient`
/// type through its `client_message` field.
#[deprecated(
    since = "0.51.0",
    note = "Misbehaviour reports should be submitted via `MsgUpdateClient` through its `client_message` field"
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgSubmitMisbehaviour {
    /// client unique identifier
    pub client_id: ClientId,
    /// misbehaviour used for freezing the light client
    pub misbehaviour: ProtoAny,
    /// signer address
    pub signer: Signer,
}

impl Protobuf<RawMsgSubmitMisbehaviour> for MsgSubmitMisbehaviour {}

impl TryFrom<RawMsgSubmitMisbehaviour> for MsgSubmitMisbehaviour {
    type Error = ClientError;

    fn try_from(raw: RawMsgSubmitMisbehaviour) -> Result<Self, Self::Error> {
        let raw_misbehaviour = raw
            .misbehaviour
            .ok_or(ClientError::MissingRawMisbehaviour)?;

        Ok(MsgSubmitMisbehaviour {
            client_id: raw
                .client_id
                .parse()
                .map_err(ClientError::InvalidRawMisbehaviour)?,
            misbehaviour: raw_misbehaviour,
            signer: raw.signer.into(),
        })
    }
}

impl From<MsgSubmitMisbehaviour> for RawMsgSubmitMisbehaviour {
    fn from(ics_msg: MsgSubmitMisbehaviour) -> Self {
        RawMsgSubmitMisbehaviour {
            client_id: ics_msg.client_id.to_string(),
            misbehaviour: Some(ics_msg.misbehaviour),
            signer: ics_msg.signer.to_string(),
        }
    }
}
