use alloc::format;
use core::str::FromStr;

use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::core::ics24_host::identifier::PortId;
use crate::Signer;

/// The default prefix for the controller port identifiers.
pub const CONTROLLER_PORT_PREFIX: &str = "interchain-account";

/// The default port identifier that the host chain typically bind with.
pub const HOST_PORT_ID: &str = "icahost";

/// Returns a new prefixed controller port identifier using the given owner string
pub fn new_controller_port_id(owner: &Signer) -> Result<PortId, InterchainAccountError> {
    if owner.as_ref().is_empty() {
        return Err(InterchainAccountError::empty("controller owner address"));
    }

    let port_id_str = format!("{}-{}", CONTROLLER_PORT_PREFIX, owner.as_ref());

    PortId::from_str(&port_id_str).map_err(InterchainAccountError::source)
}

pub fn verify_controller_port_id_prefix(port_id: &PortId) -> Result<(), InterchainAccountError> {
    if !port_id.as_str().starts_with(CONTROLLER_PORT_PREFIX) {
        return Err(InterchainAccountError::invalid("controller port id prefix")
            .expected(&format!("{CONTROLLER_PORT_PREFIX} as prefix"))
            .given(port_id));
    }

    Ok(())
}

/// Returns an instance of the default host port identifier.
pub fn default_host_port_id() -> Result<PortId, InterchainAccountError> {
    PortId::from_str(HOST_PORT_ID).map_err(InterchainAccountError::source)
}
