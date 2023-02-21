//! This module implements the processing logic for ICS2 (client abstractions and functions) msgs.

pub mod create_client;
pub mod misbehaviour;
pub mod update_client;
pub mod upgrade_client;

#[derive(Clone, Debug, PartialEq)]
pub enum ClientResult {
    Create(create_client::CreateClientResult),
    Update(update_client::UpdateClientResult),
    Upgrade(upgrade_client::UpgradeClientResult),
    Misbehaviour(misbehaviour::MisbehaviourResult),
}
