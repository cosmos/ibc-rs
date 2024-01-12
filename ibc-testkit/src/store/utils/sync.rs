use core::panic;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait Async: Send + Sync + 'static {}

impl<A> Async for A where A: Send + Sync + 'static {}

pub type SharedRw<T> = Arc<RwLock<T>>;

pub trait SharedRwExt<T> {
    fn read_access(&self) -> RwLockReadGuard<'_, T>;
    fn write_access(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> SharedRwExt<T> for SharedRw<T> {
    fn read_access(&self) -> RwLockReadGuard<'_, T> {
        match self.read() {
            Ok(guard) => guard,
            Err(poisoned) => panic!("poisoned lock: {:?}", poisoned),
        }
    }

    fn write_access(&self) -> RwLockWriteGuard<'_, T> {
        match self.write() {
            Ok(guard) => guard,
            Err(poisoned) => panic!("poisoned lock: {:?}", poisoned),
        }
    }
}
