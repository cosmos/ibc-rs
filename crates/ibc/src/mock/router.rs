use crate::{core::ics24_host::identifier::PortId, prelude::*};
use alloc::{collections::BTreeMap, sync::Arc};

use crate::core::router::{Module, ModuleId, Router};

pub struct MockRouter {
    router: BTreeMap<ModuleId, Arc<dyn Module>>,

    /// Maps ports to the the module that owns it
    pub port_to_module: BTreeMap<PortId, ModuleId>,
}

impl MockRouter {
    pub fn new() -> Self {
        Self {
            router: BTreeMap::new(),
            port_to_module: BTreeMap::new(),
        }
    }

    pub fn add_route(
        &mut self,
        module_id: ModuleId,
        module: impl Module + 'static,
    ) -> Result<(), String> {
        match self.router.insert(module_id, Arc::new(module)) {
            None => Ok(()),
            Some(_) => Err("Duplicate module_id".to_owned()),
        }
    }

    pub fn scope_port_to_module(&mut self, port_id: PortId, module_id: ModuleId) {
        self.port_to_module.insert(port_id, module_id);
    }
}

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

    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
        self.port_to_module.get(port_id).cloned()
    }
}
