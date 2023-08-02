use crate::{core::ics24_host::identifier::PortId, prelude::*};
use alloc::{collections::BTreeMap, sync::Arc};

use crate::core::router::{Module, Router};

#[derive(Default)]
pub struct MockRouter {
    /// Maps ports to the the module that owns it
    router: BTreeMap<PortId, Arc<dyn Module>>,
}

impl MockRouter {
    pub fn add_route(
        &mut self,
        port_id: PortId,
        module: impl Module + 'static,
    ) -> Result<(), String> {
        match self.router.insert(port_id, Arc::new(module)) {
            None => Ok(()),
            Some(_) => Err("Duplicate module_id".to_owned()),
        }
    }
}

impl Router for MockRouter {
    fn get_route(&self, port_id: &PortId) -> Option<&dyn Module> {
        self.router.get(port_id).map(Arc::as_ref)
    }
    fn get_route_mut(&mut self, port_id: &PortId) -> Option<&mut dyn Module> {
        // NOTE: The following:

        // self.router.get_mut(module_id).and_then(Arc::get_mut)

        // doesn't work due to a compiler bug. So we expand it out manually.

        match self.router.get_mut(port_id) {
            Some(arc_mod) => match Arc::get_mut(arc_mod) {
                Some(m) => Some(m),
                None => None,
            },
            None => None,
        }
    }
}
