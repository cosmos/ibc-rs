#![allow(deprecated)]

//! Defines the client message types that are sent to the chain by the relayer.

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;

mod create_client;
mod misbehaviour;
mod recover_client;
mod update_client;
mod upgrade_client;

pub use create_client::*;
pub use misbehaviour::*;
pub use recover_client::*;
pub use update_client::*;
pub use upgrade_client::*;

/// Encodes all the different client messages
#[allow(dead_code)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum ClientMsg {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    Misbehaviour(MsgSubmitMisbehaviour),
    UpgradeClient(MsgUpgradeClient),
    RecoverClient(MsgRecoverClient),
}

pub enum MsgUpdateOrMisbehaviour {
    UpdateClient(MsgUpdateClient),
    Misbehaviour(MsgSubmitMisbehaviour),
}

impl MsgUpdateOrMisbehaviour {
    pub fn client_id(&self) -> &ClientId {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => &msg.client_id,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => &msg.client_id,
        }
    }

    pub fn client_message(self) -> Any {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => msg.client_message,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => msg.misbehaviour,
        }
    }

    pub fn signer(&self) -> &Signer {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => &msg.signer,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => &msg.signer,
        }
    }
}
