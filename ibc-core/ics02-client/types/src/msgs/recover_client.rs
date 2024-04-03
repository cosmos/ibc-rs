//! Definition of domain type message `MsgRecoverClient`.

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::client::v1::MsgRecoverClient as RawMsgRecoverClient;
use ibc_proto::Protobuf;

use crate::error::ClientError;

pub const RECOVER_CLIENT_TYPE_URL: &str = "/ibc.core.client.v1.MsgRecoverClient";

/// Defines the message used to recover a frozen or expired client.
///
/// Note that a frozen or expired client can only be recovered by passing
/// a governance proposal. For this reason, ibc-rs does not export dispatching
/// a `MsgRecoverClient` via the `dispatch` function. In other words, the
/// client recovery functionality is not part of ibc-rs's public API. The
/// intended usage of this message type is to be integrated with hosts'
/// governance modules, not to be called directly via `dispatch`.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgRecoverClient {
    /// Client identifier of the client to be updated if the proposal passes.
    pub subject_client_id: ClientId,
    /// Client identifier of the client that will replace the subject client
    /// if the proposal passes.
    pub substitute_client_id: ClientId,
    /// The address of the signer who serves as the authority for the IBC
    /// module.
    pub signer: Signer,
}

impl Protobuf<RawMsgRecoverClient> for MsgRecoverClient {}

impl TryFrom<RawMsgRecoverClient> for MsgRecoverClient {
    type Error = ClientError;

    fn try_from(raw: RawMsgRecoverClient) -> Result<Self, Self::Error> {
        Ok(MsgRecoverClient {
            subject_client_id: raw
                .subject_client_id
                .parse()
                .map_err(ClientError::InvalidMsgRecoverClientId)?,
            substitute_client_id: raw
                .substitute_client_id
                .parse()
                .map_err(ClientError::InvalidMsgRecoverClientId)?,
            signer: raw.signer.into(),
        })
    }
}

impl From<MsgRecoverClient> for RawMsgRecoverClient {
    fn from(ics_msg: MsgRecoverClient) -> Self {
        RawMsgRecoverClient {
            subject_client_id: ics_msg.subject_client_id.to_string(),
            substitute_client_id: ics_msg.substitute_client_id.to_string(),
            signer: ics_msg.signer.to_string(),
        }
    }
}
