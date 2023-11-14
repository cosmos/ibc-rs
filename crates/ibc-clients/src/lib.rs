//! Light client implementations to be used in [core](crate::core).

use core::any::Any;

pub mod ibc_client_tendermint;

/// Allows type to be converted to `&dyn Any`
pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<M: Any> AsAny for M {
    fn as_any(&self) -> &dyn Any {
        self
    }
}