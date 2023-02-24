use super::module::{ExecutionModule, Module, ModuleId, ModuleLookup, ValidationModule};

pub trait Router: ModuleLookup {
    /// Returns a reference to a `Module` registered against the specified `ModuleId`
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

    /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;
}

pub trait ValidationRouter: ModuleLookup {
    /// Returns a reference to a `ValidationModule` registered against the specified `ModuleId`
    fn get_validation_route(&self, module_id: &ModuleId) -> Option<&dyn ValidationModule>;

    /// Returns true if the `ValidationRouter` has a `ValidationModule` registered against the specified `ModuleId`
    fn has_validation_route(&self, module_id: &ModuleId) -> bool {
        self.get_validation_route(module_id).is_some()
    }
}

pub trait ExecutionRouter: ModuleLookup {
    /// Returns a mutable reference to a `ExecutionModule` registered against the specified `ModuleId`
    fn get_execution_route(&mut self, module_id: &ModuleId) -> Option<&mut dyn ExecutionModule>;

    /// Returns true if the `ExecutionRouter` has a `ExecutionModule` registered against the specified `ModuleId`
    fn has_execution_route(&mut self, module_id: &ModuleId) -> bool {
        self.get_execution_route(module_id).is_some()
    }
}

impl<T> ValidationRouter for T
where
    T: Router,
{
    fn get_validation_route(&self, module_id: &ModuleId) -> Option<&dyn ValidationModule> {
        self.get_route(module_id)
            .map(|module| module.as_validation_module())
    }
}

impl<T> ExecutionRouter for T
where
    T: Router,
{
    fn get_execution_route(&mut self, module_id: &ModuleId) -> Option<&mut dyn ExecutionModule> {
        self.get_route_mut(module_id)
            .map(|module| module.as_execution_module())
    }
}
