//! Defines the client message types that are sent to the chain by the relayer.
use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::msgs::create_client::MsgCreateClient;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics24_host::identifier::ClientId;
use crate::prelude::*;
use crate::signer::Signer;

pub mod create_client;
pub mod misbehaviour;
pub mod update_client;
pub mod upgrade_client;

/// Encodes all the different client messages
#[allow(dead_code)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMsg {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    Misbehaviour(MsgSubmitMisbehaviour),
    UpgradeClient(MsgUpgradeClient),
}

pub(crate) enum MsgUpdateOrMisbehaviour {
    UpdateClient(MsgUpdateClient),
    Misbehaviour(MsgSubmitMisbehaviour),
}

impl MsgUpdateOrMisbehaviour {
    pub(crate) fn client_id(&self) -> &ClientId {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => &msg.client_id,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => &msg.client_id,
        }
    }

    pub(crate) fn client_message(self) -> Any {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => msg.client_message,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => msg.misbehaviour,
        }
    }

    pub(crate) fn signer(&self) -> &Signer {
        match self {
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => &msg.signer,
            MsgUpdateOrMisbehaviour::Misbehaviour(msg) => &msg.signer,
        }
    }
}
