#[cfg(feature = "serde")]
pub use erased_serde::Serialize as ErasedSerialize;
#[cfg(not(feature = "serde"))]
pub trait ErasedSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> ErasedSerialize for T {}
