//! Definition of domain type message `MsgProvideCounterParty`.

use ibc_core_commitment_types::commitment::CommitmentPrefix;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;

pub const PROVIDE_COUNTER_PARTY_TYPE_URL: &str = "/ibc.core.client.v1.MsgProvideCounterParty";

/// Defines the message used to provide the client identifier at counter party.
///
/// Note that a counter party client can only be provided by passing
/// a governance proposal. For this reason, ibc-rs does not export dispatching
/// a `MsgProvideCounterParty` via the `dispatch` function. In other words, this
/// functionality is not part of ibc-rs's public API. The
/// intended usage of this message type is to be integrated with hosts'
/// governance modules, not to be called directly via `dispatch`.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgProvideCounterParty {
    /// Client identifier of the host client id.
    pub client_id: ClientId,
    /// Client identifier of the counter party client id.
    pub counter_party_client_id: ClientId,
    /// Commitment prefix of the counter party.
    pub prefix: CommitmentPrefix,
    /// The address of the signer who serves as the authority for the IBC
    /// module.
    pub signer: Signer,
}

// impl Protobuf<RawMsgProvideCounterParty> for MsgProvideCounterParty {}

// impl TryFrom<RawMsgProvideCounterParty> for MsgProvideCounterParty {
//     type Error = DecodingError;

//     fn try_from(raw: RawMsgProvideCounterParty) -> Result<Self, Self::Error> {
//         Ok(MsgProvideCounterParty {
//             signer: raw.signer.into(),
//         })
//     }
// }

// impl From<MsgProvideCounterParty> for RawMsgProvideCounterParty {
//     fn from(ics_msg: MsgProvideCounterParty) -> Self {
//         RawMsgProvideCounterParty {
//             signer: ics_msg.signer.to_string(),
//         }
//     }
// }
