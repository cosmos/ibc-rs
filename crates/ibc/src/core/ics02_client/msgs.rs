//! These are definitions of messages that a relayer submits to a chain. Specific implementations of
//! these messages can be found, for instance, in ICS 07 for Tendermint-specific chains. A chain
//! handles these messages in two layers: first with the general ICS 02 client handler, which
//! subsequently calls into the chain-specific (e.g., ICS 07) client handler. See:
//! <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics#create>.

use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::msgs::create_client::MsgCreateClient;
use crate::core::ics02_client::msgs::misbehaviour::MsgSubmitMisbehaviour;
use crate::core::ics02_client::msgs::update_client::MsgUpdateClient;
use crate::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::Signer;

pub mod create_client;
pub mod misbehaviour;
pub mod update_client;
pub mod upgrade_client;

/// Encodes all the different client messages
#[allow(dead_code)]
#[derive(Clone, Debug)]
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
            MsgUpdateOrMisbehaviour::UpdateClient(msg) => msg.header,
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
