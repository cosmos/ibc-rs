use crate::prelude::*;

use ibc_proto::google::protobuf::Any as ProtoAny;
use ibc_proto::ibc::core::client::v1::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::Signer;
use crate::tx_msg::Msg;

pub const TYPE_URL: &str = "/ibc.core.client.v1.MsgSubmitMisbehaviour";

/// A type of message that submits client misbehaviour proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgSubmitMisbehaviour {
    /// client unique identifier
    pub client_id: ClientId,
    /// misbehaviour used for freezing the light client
    pub misbehaviour: ProtoAny,
    /// signer address
    pub signer: Signer,
}

impl Msg for MsgSubmitMisbehaviour {
    type Raw = RawMsgSubmitMisbehaviour;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
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
            signer: raw.signer.parse().map_err(ClientError::Signer)?,
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
