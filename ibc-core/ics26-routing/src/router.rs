//! Defines the `Router`, which binds modules to ports

use ibc_core_host_types::identifiers::PortId;
use ibc_core_router_types::module::ModuleId;

use crate::module::Module;

/// Router as defined in ICS-26, which binds modules to ports.
pub trait Router {
    /// Returns a reference to a `Module` registered against the specified `ModuleId`
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

    /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;

    /// Return the module_id associated with a given port_id
    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId>;
}
