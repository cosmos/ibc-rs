use alloc::sync::Arc;

use ibc::core::host::types::identifiers::PortId;
use ibc::core::router::module::Module;
use ibc::core::router::router::Router;
use ibc::core::router::types::module::ModuleId;

use super::types::MockRouter;

impl Router for MockRouter {
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        self.router.get(module_id).map(Arc::as_ref)
    }
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        // NOTE: The following:

        // self.router.get_mut(module_id).and_then(Arc::get_mut)

        // doesn't work due to a compiler bug. So we expand it out manually.

        match self.router.get_mut(module_id) {
            Some(arc_mod) => match Arc::get_mut(arc_mod) {
                Some(m) => Some(m),
                None => None,
            },
            None => None,
        }
    }

    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
        self.port_to_module.get(port_id).cloned()
    }
}
