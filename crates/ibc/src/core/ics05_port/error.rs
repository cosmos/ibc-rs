use crate::core::ics24_host::identifier::PortId;
use alloc::string::String;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum PortError {
    /// port `{port_id}` is unknown
    UnknownPort { port_id: PortId },
    /// port `{port_id}` is already bound to `{port_id_owner}`
    PortAlreadyBound {
        port_id: PortId,
        port_id_owner: String,
    },
    /// port `{port_id}` is not bound
    PortNotBound { port_id: PortId },
    /// implementation specific error
    ImplementationSpecific,
}

#[cfg(feature = "std")]
impl std::error::Error for PortError {}
