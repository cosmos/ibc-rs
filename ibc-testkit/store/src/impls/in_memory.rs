use ics23::CommitmentProof;
use tendermint::hash::Algorithm;
use tendermint::Hash;
use tracing::trace;

use crate::avl::{AsBytes, AvlTree};
use crate::context::{ProvableStore, Store};
use crate::types::{Height, Path, State};

/// An in-memory store backed by an AvlTree.
#[derive(Clone, Debug)]
pub struct InMemoryStore {
    /// collection of states corresponding to every committed block height
    store: Vec<State>,
    /// pending block state
    pending: State,
}

impl InMemoryStore {
    #[inline]
    fn get_state(&self, height: Height) -> Option<&State> {
        match height {
            Height::Pending => Some(&self.pending),
            Height::Latest => self.store.last(),
            Height::Stable(height) => {
                let h = height as usize;
                if h <= self.store.len() {
                    self.store.get(h - 1)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for InMemoryStore {
    /// The store starts out with an empty state. We also initialize the pending location as empty.
    fn default() -> Self {
        Self {
            store: vec![],
            pending: AvlTree::new(),
        }
    }
}

impl Store for InMemoryStore {
    type Error = (); // underlying store ops are infallible

    fn set(&mut self, path: Path, value: Vec<u8>) -> Result<Option<Vec<u8>>, Self::Error> {
        trace!("set at path = {}", path.to_string());
        Ok(self.pending.insert(path, value))
    }

    fn get(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        trace!(
            "get at path = {} at height = {:?}",
            path.to_string(),
            height
        );
        self.get_state(height).and_then(|v| v.get(path).cloned())
    }

    fn delete(&mut self, _path: &Path) {
        todo!()
    }

    fn commit(&mut self) -> Result<Vec<u8>, Self::Error> {
        trace!("committing height: {}", self.store.len());
        self.store.push(self.pending.clone());
        Ok(self.root_hash())
    }

    fn current_height(&self) -> u64 {
        self.store.len() as u64
    }

    fn get_keys(&self, key_prefix: &Path) -> Vec<Path> {
        let key_prefix = key_prefix.as_bytes();
        self.pending
            .get_keys()
            .into_iter()
            .filter(|&key| key.as_bytes().as_ref().starts_with(key_prefix.as_ref()))
            .cloned()
            .collect()
    }
}

impl ProvableStore for InMemoryStore {
    fn root_hash(&self) -> Vec<u8> {
        self.pending
            .root_hash()
            .unwrap_or(&Hash::from_bytes(Algorithm::Sha256, &[0u8; 32]).unwrap())
            .as_bytes()
            .to_vec()
    }

    fn get_proof(&self, height: Height, key: &Path) -> Option<CommitmentProof> {
        trace!(
            "get proof at path = {} at height = {:?}",
            key.to_string(),
            height
        );
        self.get_state(height).and_then(|v| v.get_proof(key))
    }
}

// TODO(hu55a1n1): import tests
