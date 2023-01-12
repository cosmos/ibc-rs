use crate::prelude::*;

use dyn_clone::DynClone;
use ibc_proto::google::protobuf::Any;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;

use crate::core::ics02_client::error::ClientError;
use crate::dynamic_typing::AsAny;
use crate::erased::ErasedSerialize;
use crate::timestamp::Timestamp;
use crate::Height;

/// Abstract of consensus state update information
///
/// Users are not expected to implement sealed::ErasedPartialEqHeader.
/// Effectively, that trait bound mandates implementors to derive PartialEq,
/// after which our blanket implementation will implement
/// `ErasedPartialEqHeader` for their type.
pub trait Header:
    AsAny
    + sealed::ErasedPartialEqHeader
    + DynClone
    + ErasedSerialize
    + ErasedProtobuf<Any, Error = ClientError>
    + core::fmt::Debug
    + Send
    + Sync
{
    /// The height of the consensus state
    fn height(&self) -> Height;

    /// The timestamp of the consensus state
    fn timestamp(&self) -> Timestamp;

    /// Convert into a boxed trait object
    fn into_box(self) -> Box<dyn Header>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// Implements `Clone` for `Box<dyn Header>`
dyn_clone::clone_trait_object!(Header);

// Implements `serde::Serialize` for all types that have Header as supertrait
#[cfg(feature = "serde")]
erased_serde::serialize_trait_object!(Header);

pub fn downcast_header<H: Header>(h: &dyn Header) -> Option<&H> {
    h.as_any().downcast_ref::<H>()
}

impl PartialEq for dyn Header {
    fn eq(&self, other: &Self) -> bool {
        self.eq_header(other)
    }
}

mod sealed {
    use super::*;

    pub trait ErasedPartialEqHeader {
        fn eq_header(&self, other: &dyn Header) -> bool;
    }

    impl<H> ErasedPartialEqHeader for H
    where
        H: Header + PartialEq,
    {
        fn eq_header(&self, other: &dyn Header) -> bool {
            other
                .as_any()
                .downcast_ref::<H>()
                .map_or(false, |h| self == h)
        }
    }
}
