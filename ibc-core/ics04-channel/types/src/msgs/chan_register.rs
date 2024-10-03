use ibc_core_commitment_types::commitment::CommitmentPrefix;
use ibc_core_host_types::identifiers::{ClientId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;

use crate::channel::Order;
use crate::Version;

pub const CHAN_REGISTER_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelRegister";

#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelRegister {
    pub port_id_on_a: PortId,
    pub client_id_on_a: ClientId,
    pub port_id_on_b: PortId,
    pub client_id_on_b: ClientId,
    pub commitment_prefix_on_b: CommitmentPrefix,
    pub ordering: Order,
    pub signer: Signer,
    /// Allow a relayer to specify a particular version by providing a non-empty version string
    pub version_proposal: Version,
}
