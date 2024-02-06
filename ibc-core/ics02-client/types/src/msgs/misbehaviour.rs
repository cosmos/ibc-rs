//! Definition of domain type message `MsgSubmitMisbehaviour`.

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any as ProtoAny;

pub const SUBMIT_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.core.client.v1.MsgSubmitMisbehaviour";

/// A type of message that submits client misbehaviour proof.
///
/// Deprecated since v0.50.0. Misbehaviour reports should be submitted via the `MsgUpdateClient`
/// type through its `client_message` field.
#[deprecated(
    since = "0.50.0",
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
