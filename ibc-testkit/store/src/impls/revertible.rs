use crate::context::ProvableStore;
use crate::context::Store;
use crate::types::Height;
use crate::types::Path;

use ics23::CommitmentProof;
use tracing::trace;

/// A wrapper store that implements rudimentary `apply()`/`reset()` support for other stores
#[derive(Clone, Debug)]
pub struct RevertibleStore<S> {
    /// backing store
    store: S,
    /// operation log for recording rollback operations in preserved order
    op_log: Vec<RevertOp>,
}

#[derive(Clone, Debug)]
enum RevertOp {
    Delete(Path),
    Set(Path, Vec<u8>),
}

impl<S> RevertibleStore<S>
where
    S: Store,
{
    pub fn new(store: S) -> Self {
        Self {
            store,
            op_log: vec![],
        }
    }
}

impl<S> Default for RevertibleStore<S>
where
    S: Default + Store,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

impl<S> Store for RevertibleStore<S>
where
    S: Store,
{
    type Error = S::Error;

    #[inline]
    fn set(&mut self, path: Path, value: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        let old_value = self.store.set(path.clone(), value)?;
        match old_value {
            // None implies this was an insert op, so we record the revert op as delete op
            None => self.op_log.push(RevertOp::Delete(path)),
            // Some old value implies this was an update op, so we record the revert op as a set op
            // with the old value
            Some(ref old_value) => self.op_log.push(RevertOp::Set(path, old_value.clone())),
        }
        Ok(old_value)
    }

    #[inline]
    fn get(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        self.store.get(height, path)
    }

    #[inline]
    fn delete(&mut self, path: &Path) {
        self.store.delete(path)
    }

    #[inline]
    fn commit(&mut self) -> Result<Vec<u8>, Self::Error> {
        // call `apply()` before `commit()` to make sure all operations are applied
        self.apply()?;
        self.store.commit()
    }

    #[inline]
    fn apply(&mut self) -> Result<(), Self::Error> {
        // note that we do NOT call the backing store's apply here - this allows users to create
        // multilayered `WalStore`s
        self.op_log.clear();
        Ok(())
    }

    #[inline]
    fn reset(&mut self) {
        // note that we do NOT call the backing store's reset here - this allows users to create
        // multilayered `WalStore`s
        trace!("Rollback operation log changes");
        while let Some(op) = self.op_log.pop() {
            match op {
                RevertOp::Delete(path) => self.delete(&path),
                RevertOp::Set(path, value) => {
                    // FIXME: potential non-termination
                    // self.set() may insert a new op into the op_log
                    self.set(path, value).unwrap(); // safety - reset failures are unrecoverable
                }
            }
        }
    }

    #[inline]
    fn current_height(&self) -> u64 {
        self.store.current_height()
    }

    #[inline]
    fn get_keys(&self, key_prefix: &Path) -> Vec<Path> {
        self.store.get_keys(key_prefix)
    }
}

impl<S> ProvableStore for RevertibleStore<S>
where
    S: ProvableStore,
{
    #[inline]
    fn root_hash(&self) -> Vec<u8> {
        self.store.root_hash()
    }

    #[inline]
    fn get_proof(&self, height: Height, key: &Path) -> Option<CommitmentProof> {
        self.store.get_proof(height, key)
    }
}
