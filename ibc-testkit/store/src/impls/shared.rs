use crate::context::{ProvableStore, Store};
use crate::types::{Height, Path, RawHeight};
use crate::utils::{SharedRw, SharedRwExt};

use ics23::CommitmentProof;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

/// Wraps a store to make it shareable by cloning
#[derive(Clone, Debug)]
pub struct SharedStore<S>(SharedRw<S>);

impl<S> SharedStore<S> {
    pub fn new(store: S) -> Self {
        Self(Arc::new(RwLock::new(store)))
    }

    pub fn share(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S> Default for SharedStore<S>
where
    S: Default + Store,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

impl<S> Store for SharedStore<S>
where
    S: Store,
{
    type Error = S::Error;

    #[inline]
    fn set(&mut self, path: Path, value: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        self.write_access().set(path, value)
    }

    #[inline]
    fn get(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        self.read_access().get(height, path)
    }

    #[inline]
    fn delete(&mut self, path: &Path) {
        self.write_access().delete(path)
    }

    #[inline]
    fn commit(&mut self) -> Result<Vec<u8>, Self::Error> {
        self.write_access().commit()
    }

    #[inline]
    fn apply(&mut self) -> Result<(), Self::Error> {
        self.write_access().apply()
    }

    #[inline]
    fn reset(&mut self) {
        self.write_access().reset()
    }

    #[inline]
    fn current_height(&self) -> RawHeight {
        self.read_access().current_height()
    }

    #[inline]
    fn get_keys(&self, key_prefix: &Path) -> Vec<Path> {
        self.read_access().get_keys(key_prefix)
    }
}

impl<S> ProvableStore for SharedStore<S>
where
    S: ProvableStore,
{
    #[inline]
    fn root_hash(&self) -> Vec<u8> {
        self.read_access().root_hash()
    }

    #[inline]
    fn get_proof(&self, height: Height, key: &Path) -> Option<CommitmentProof> {
        self.read_access().get_proof(height, key)
    }
}

impl<S> Deref for SharedStore<S> {
    type Target = Arc<RwLock<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for SharedStore<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
