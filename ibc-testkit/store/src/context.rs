use crate::types::{Height, Path, RawHeight};
use crate::utils::Async;

use ics23::CommitmentProof;
use std::fmt::Debug;

/// Store trait - maybe provableStore or privateStore
pub trait Store: Async + Clone {
    /// Error type - expected to envelope all possible errors in store
    type Error: Debug;

    /// Set `value` for `path`
    fn set(&mut self, path: Path, value: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Get associated `value` for `path` at specified `height`
    fn get(&self, height: Height, path: &Path) -> Option<Vec<u8>>;

    /// Delete specified `path`
    // TODO(rano): return Result to denote success or failure
    fn delete(&mut self, path: &Path);

    /// Commit `Pending` block to canonical chain and create new `Pending`
    fn commit(&mut self) -> Result<Vec<u8>, Self::Error>;

    /// Apply accumulated changes to `Pending`
    fn apply(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Reset accumulated changes
    fn reset(&mut self) {}

    /// Prune historic blocks upto specified `height`
    fn prune(&mut self, height: RawHeight) -> Result<RawHeight, Self::Error> {
        Ok(height)
    }

    /// Return the current height of the chain
    fn current_height(&self) -> RawHeight;

    /// Return all keys that start with specified prefix
    fn get_keys(&self, key_prefix: &Path) -> Vec<Path>; // TODO(hu55a1n1): implement support for all heights
}

/// ProvableStore trait
pub trait ProvableStore: Store {
    /// Return a vector commitment
    fn root_hash(&self) -> Vec<u8>;

    /// Return proof of existence for key
    fn get_proof(&self, height: Height, key: &Path) -> Option<CommitmentProof>;
}
