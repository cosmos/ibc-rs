use crate::core::ics24_host::identifier::PortId;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum PortError {
    /// port `{port_id}` is unknown
    UnknownPort { port_id: PortId },
    /// implementation specific error
    ImplementationSpecific,
}

#[cfg(feature = "std")]
impl std::error::Error for PortError {}
