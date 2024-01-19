use crate::avl::AvlTree;
use crate::context::Store;
use crate::impls::{RevertibleStore, SharedStore};
use crate::types::{Height, Path, RawHeight};
use crate::utils::codec::{BinCodec, JsonCodec, NullCodec, ProtobufCodec};
use crate::utils::Codec;
use std::{fmt::Debug, marker::PhantomData};

// A state type that represents a snapshot of the store at every block.
// The value is a `Vec<u8>` to allow stored types to choose their own serde.
pub type State = AvlTree<Path, Vec<u8>>;

pub type MainStore<S> = SharedStore<RevertibleStore<S>>;

/// A `TypedStore` that uses the `JsonCodec`
pub type JsonStore<S, K, V> = TypedStore<S, K, JsonCodec<V>>;

/// A `TypedStore` that uses the `ProtobufCodec`
pub type ProtobufStore<S, K, V, R> = TypedStore<S, K, ProtobufCodec<V, R>>;

/// A `TypedSet` that stores only paths and no values
pub type TypedSet<S, K> = TypedStore<S, K, NullCodec>;

/// A `TypedStore` that uses the `BinCodec`
pub type BinStore<S, K, V> = TypedStore<S, K, BinCodec<V>>;

#[derive(Clone, Debug)]
pub struct TypedStore<S, K, C> {
    store: S,
    _key: PhantomData<K>,
    _codec: PhantomData<C>,
}

impl<S, K, C, V> TypedStore<S, K, C>
where
    S: Store,
    C: Codec<Type = V>,
    K: Into<Path> + Clone,
{
    #[inline]
    pub fn new(store: S) -> Self {
        Self {
            store,
            _codec: PhantomData,
            _key: PhantomData,
        }
    }

    #[inline]
    pub fn set(&mut self, path: K, value: V) -> Result<Option<V>, S::Error> {
        self.store
            .set(path.into(), C::encode(&value).unwrap().as_ref().to_vec())
            .map(|prev_val| prev_val.and_then(|v| C::decode(&v)))
    }

    #[inline]
    pub fn delete(&mut self, path: K) {
        self.store.delete(&path.into())
    }

    #[inline]
    pub fn get(&self, height: Height, path: &K) -> Option<V> {
        self.store
            .get(height, &path.clone().into())
            .and_then(|v| C::decode(&v))
    }

    #[inline]
    pub fn get_keys(&self, key_prefix: &Path) -> Vec<Path> {
        self.store.get_keys(key_prefix)
    }

    #[inline]
    pub fn current_height(&self) -> RawHeight {
        self.store.current_height()
    }
}

impl<S, K> TypedStore<S, K, NullCodec>
where
    S: Store,
    K: Into<Path> + Clone,
{
    #[inline]
    pub fn set_path(&mut self, path: K) -> Result<(), S::Error> {
        self.store
            .set(path.into(), NullCodec::encode(&()).unwrap())
            .map(|_| ())
    }

    #[inline]
    pub fn is_path_set(&self, height: Height, path: &K) -> bool {
        self.store.get(height, &path.clone().into()).is_some()
    }
}
