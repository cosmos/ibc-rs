use crate::context::ProvableStore;
use crate::context::Store;
use crate::types::Height;
use crate::types::Path;

use ics23::CommitmentProof;

/// GrowingStore does not prune any path.
/// If the path is set to v, the stored value is v
/// If the path is deleted, the stored value is []
/// Note: we should not allow empty vec to store as
/// this would conflict with the deletion representation.
#[derive(Clone, Debug)]
pub struct GrowingStore<S> {
    store: S,
}

impl<S> GrowingStore<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> Default for GrowingStore<S>
where
    S: Default,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

impl<S> Store for GrowingStore<S>
where
    S: Store,
{
    type Error = S::Error;

    #[inline]
    fn set(&mut self, path: Path, value: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        if value.is_empty() {
            panic!("empty vec is not allowed to store")
        }
        self.store.set(path, value)
    }

    #[inline]
    fn get(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        // ignore if path is deleted
        self.store.get(height, path).filter(|v| !v.is_empty())
    }

    #[inline]
    fn delete(&mut self, path: &Path) {
        // set value to empty vec to denote the path is deleted.
        self.store.set(path.clone(), vec![]).expect("delete failed");
    }

    fn commit(&mut self) -> Result<Vec<u8>, Self::Error> {
        self.store.commit()
    }

    #[inline]
    fn apply(&mut self) -> Result<(), Self::Error> {
        self.store.apply()
    }

    #[inline]
    fn reset(&mut self) {
        self.store.reset()
    }

    #[inline]
    fn prune(&mut self, height: u64) -> Result<u64, Self::Error> {
        self.store.prune(height)
    }

    #[inline]
    fn current_height(&self) -> u64 {
        self.store.current_height()
    }

    #[inline]
    fn get_keys(&self, key_prefix: &Path) -> Vec<Path> {
        self.store
            .get_keys(key_prefix)
            .into_iter()
            // ignore the deleted paths
            .filter(|k| {
                self.get(Height::Pending, k)
                    .filter(|v| !v.is_empty())
                    .is_some()
            })
            .collect()
    }
}

impl<S> ProvableStore for GrowingStore<S>
where
    S: ProvableStore,
{
    #[inline]
    fn root_hash(&self) -> Vec<u8> {
        self.store.root_hash()
    }

    #[inline]
    fn get_proof(&self, height: Height, key: &Path) -> Option<CommitmentProof> {
        self.get(height, key)
            // ignore if path is deleted
            .filter(|v| !v.is_empty())
            .and_then(|_| self.store.get_proof(height, key))
    }
}

impl<S> GrowingStore<S>
where
    S: Store,
{
    #[inline]
    pub fn is_deleted(&self, path: &Path) -> bool {
        self.get(Height::Pending, path)
            .filter(|v| v.is_empty())
            .is_some()
    }

    #[inline]
    pub fn deleted_keys(&self, key_prefix: &Path) -> Vec<Path> {
        self.store
            .get_keys(key_prefix)
            .into_iter()
            .filter(|k| {
                self.get(Height::Pending, k)
                    .filter(|v| v.is_empty())
                    .is_some()
            })
            .collect()
    }
}
