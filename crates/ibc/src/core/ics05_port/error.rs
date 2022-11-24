use crate::core::ics24_host::identifier::PortId;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// port `{port_id}` is unknown
    UnknownPort { port_id: PortId },
    /// port `{port_id}` is already bound
    PortAlreadyBound { port_id: PortId },
    /// could not retrieve module from port `{port_id}`
    ModuleNotFound { port_id: PortId },
    /// implementation specific error
    ImplementationSpecific,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
