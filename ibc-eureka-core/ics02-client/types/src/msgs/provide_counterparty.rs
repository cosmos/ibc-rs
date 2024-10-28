//! Definition of domain type message `MsgProvideCouterparty`.

use ibc_eureka_core_commitment_types::commitment::CommitmentPrefix;
use ibc_eureka_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;

pub const _PROVIDE_COUNTERPARTY_TYPE_URL: &str = "/ibc.core.client.v1.MsgProvideCouterparty";

/// A type of message that links an on-chain (IBC) client to its counterparty.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgProvideCouterparty {
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_commitment_prefix: CommitmentPrefix,
    pub signer: Signer,
}

impl MsgProvideCouterparty {
    pub fn new(
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_commitment_prefix: CommitmentPrefix,
        signer: Signer,
    ) -> Self {
        MsgProvideCouterparty {
            client_id,
            counterparty_client_id,
            counterparty_commitment_prefix,
            signer,
        }
    }
}
