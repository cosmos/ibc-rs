//! Light client implementations to be used in [Core](core).
//!
//! [core]: https://github.com/cosmos/ibc-rs/tree/main/crates/ibc/src/core

use core::any::Any;

pub mod ics07_tendermint;

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<M: Any> AsAny for M {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
