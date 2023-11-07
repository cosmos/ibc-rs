//! Definition of domain type message `MsgUpdateClient`.

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
use ibc_proto::Protobuf;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::Msg;
use crate::prelude::*;
use crate::signer::Signer;

pub(crate) const TYPE_URL: &str = "/ibc.core.client.v1.MsgUpdateClient";

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

impl Msg for MsgUpdateClient {
    type Raw = RawMsgUpdateClient;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
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

#[cfg(test)]
mod tests {
    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
    use test_log::test;

    use super::*;
    use crate::clients::ics07_tendermint::header::test_util::get_dummy_ics07_header;
    use crate::core::ics02_client::msgs::MsgUpdateClient;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::signer::Signer;
    use crate::test_utils::get_dummy_account_id;

    impl MsgUpdateClient {
        pub fn new(client_id: ClientId, client_message: Any, signer: Signer) -> Self {
            MsgUpdateClient {
                client_id,
                client_message,
                signer,
            }
        }
    }

    #[test]
    fn msg_update_client_serialization() {
        let client_id: ClientId = "tendermint".parse().unwrap();
        let signer = get_dummy_account_id();

        let header = get_dummy_ics07_header();

        let msg = MsgUpdateClient::new(client_id, header.into(), signer);
        let raw = RawMsgUpdateClient::from(msg.clone());
        let msg_back = MsgUpdateClient::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgUpdateClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
