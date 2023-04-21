use alloc::string::ToString;

use crate::core::ics02_client::client_type::ClientType;

pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod header;
pub mod misbehaviour;
pub mod types;

pub(crate) const SOLOMACHINE_CLIENT_TYPE: &str = "07-solomachine";

pub fn client_type() -> ClientType {
    ClientType::new(SOLOMACHINE_CLIENT_TYPE.to_string())
}
