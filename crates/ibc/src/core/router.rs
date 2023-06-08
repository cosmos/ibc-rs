//! Defines the `Router`, which binds modules to ports

use crate::prelude::*;
use alloc::{boxed::Box, string::String};

use crate::core::ics24_host::path::PortPath;
use crate::core::{ContextError, ExecutionContext, ValidationContext};

use super::module::{Module, ModuleId};

pub trait RouterRef {
    /// Returns a reference to a `Module` registered against the specified `ModuleId`
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

    /// Return the module_id associated with a given port_id
    fn lookup_module(&self, port_path: &PortPath) -> Result<ModuleId, ContextError>;
}

pub trait RouterMut {
    /// Registers a `Module` against the specified `ModuleId`
    fn add_route(&mut self, module_id: ModuleId, module: Box<dyn Module>) -> Result<(), String>;

    /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;
}

pub trait Router: RouterRef + RouterMut {}

impl<R> Router for R where R: RouterRef + RouterMut {}

impl<Ctx> RouterRef for Ctx
where
    Ctx: ValidationContext,
{
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        self.router().get_route(module_id)
    }

    fn lookup_module(&self, port_path: &PortPath) -> Result<ModuleId, ContextError> {
        self.router().lookup_module(port_path)
    }
}

impl<Ctx> RouterMut for Ctx
where
    Ctx: ExecutionContext,
{
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        self.router_mut().get_route_mut(module_id)
    }

    fn add_route(&mut self, module_id: ModuleId, module: Box<dyn Module>) -> Result<(), String> {
        self.router_mut().add_route(module_id, module)
    }
}
