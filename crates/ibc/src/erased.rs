/// If the `serde` feature is enabled then re-export `erased_serde::Serialize` as `ErasedSerialize`,
/// otherwise define an empty `ErasedSerialize` trait and provide a blanket implementation for all
/// types.
#[cfg(feature = "serde")]
pub use erased_serde::Serialize as ErasedSerialize;
#[cfg(not(feature = "serde"))]
pub trait ErasedSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> ErasedSerialize for T {}
