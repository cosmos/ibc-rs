//! Definition of domain type message `MsgUpdateAnyClient`.

use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgSubmitMisbehaviour as RawMsgSubmitMisbehaviour;
use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::Signer;

pub const UPDATE_CLIENT_TYPE_URL: &str = "/ibc.core.client.v1.MsgUpdateClient";
pub const MISBEHAVIOUR_TYPE_URL: &str = "/ibc.core.client.v1.MsgSubmitMisbehaviour";

/// `UpdateKind` represents the 2 ways that a client can be updated
/// in IBC: either through a `MsgUpdateClient`, or a `MsgSubmitMisbehaviour`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateKind {
    /// this is the typical scenario where a new header is submitted to the client
    /// to update the client. Note that light clients are free to define the type
    /// of the object used to update them (e.g. could be a list of headers).
    UpdateClient,
    /// this is the scenario where misbehaviour is submitted to the client
    /// (e.g 2 headers with the same height in Tendermint)
    SubmitMisbehaviour,
}

/// Represents the message that triggers the update of an on-chain (IBC) client
/// either with new headers, or evidence of misbehaviour.
/// Note that some types of misbehaviour can be detected when a headers
/// are updated (`UpdateKind::UpdateClient`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Any,
    pub update_kind: UpdateKind,
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
            client_message: raw.header.ok_or(ClientError::MissingRawHeader)?,
            update_kind: UpdateKind::UpdateClient,
            signer: raw.signer.parse().map_err(ClientError::Signer)?,
        })
    }
}

impl From<MsgUpdateClient> for RawMsgUpdateClient {
    fn from(ics_msg: MsgUpdateClient) -> Self {
        RawMsgUpdateClient {
            client_id: ics_msg.client_id.to_string(),
            header: Some(ics_msg.client_message),
            signer: ics_msg.signer.to_string(),
        }
    }
}

impl Protobuf<RawMsgSubmitMisbehaviour> for MsgUpdateClient {}

impl TryFrom<RawMsgSubmitMisbehaviour> for MsgUpdateClient {
    type Error = ClientError;

    fn try_from(raw: RawMsgSubmitMisbehaviour) -> Result<Self, Self::Error> {
        let raw_misbehaviour = raw
            .misbehaviour
            .ok_or(ClientError::MissingRawMisbehaviour)?;

        Ok(MsgUpdateClient {
            client_id: raw
                .client_id
                .parse()
                .map_err(ClientError::InvalidRawMisbehaviour)?,
            client_message: raw_misbehaviour,
            update_kind: UpdateKind::SubmitMisbehaviour,
            signer: raw.signer.parse().map_err(ClientError::Signer)?,
        })
    }
}

impl From<MsgUpdateClient> for RawMsgSubmitMisbehaviour {
    fn from(ics_msg: MsgUpdateClient) -> Self {
        RawMsgSubmitMisbehaviour {
            client_id: ics_msg.client_id.to_string(),
            misbehaviour: Some(ics_msg.client_message),
            signer: ics_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    use ibc_proto::google::protobuf::Any;
    use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;

    use crate::clients::ics07_tendermint::header::test_util::get_dummy_ics07_header;
    use crate::core::ics02_client::msgs::MsgUpdateClient;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::signer::Signer;
    use crate::test_utils::get_dummy_account_id;

    impl MsgUpdateClient {
        pub fn new(client_id: ClientId, header: Any, signer: Signer) -> Self {
            MsgUpdateClient {
                client_id,
                client_message: header,
                update_kind: UpdateKind::UpdateClient,
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
