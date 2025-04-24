use std::hash::Hash;
use std::{borrow::Borrow, marker::PhantomData};

use salvo_core::Depot;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{RateGuard, RateStore};

/// A simple in-memory store for rate limiter.
#[derive(Debug)]
pub struct WorkerStore<K, G>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    G: RateGuard,
{
    key: String,
    _tag_k: PhantomData<K>,
    _tag_g: PhantomData<G>,
}
impl<K, G> WorkerStore<K, G>
where
    K: Hash + Eq + Send + Sync + Clone + 'static + AsRef<str>,
    G: RateGuard,
{
    /// Create a new `WorkerStore`.
    pub fn new(key: String) -> Self {
        Self {
            key,
            _tag_k: PhantomData,
            _tag_g: PhantomData,
        }
    }
}

impl<K, G> WorkerStore<K, G>
where
    K: Hash + Eq + Send + Sync + Clone + 'static + AsRef<str>,
    G: RateGuard + Serialize + DeserializeOwned,
{
    #[worker::send]
    async fn get<Q>(&self, depot: &Depot, key: &Q) -> Option<G>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + Sync + AsRef<str>,
    {
        let env = depot.obtain::<worker::Env>().ok()?;
        let kv = env.kv(&self.key).ok()?;
        let name = key.as_ref();
        let builder = kv.get(name);
        let bytes = builder.bytes().await.ok()??;

        let guard = ciborium::de::from_reader(&bytes[..]).ok()?;
        Some(guard)
    }
    #[worker::send]
    async fn set(&self, depot: &Depot, key: K, guard: G) -> Result<(), worker::Error> {
        let env = depot
            .obtain::<worker::Env>()
            .map_err(|_| worker::Error::Json(("obtain Env failed".to_string(), 1)))?;
        let kv = env.kv(&self.key)?;

        let name = key.as_ref();
        let mut bytes = vec![];
        ciborium::ser::into_writer(&guard, &mut bytes).map_err(|err| worker::Error::Json((format!("{err:?}"), 1)))?;

        let builder = kv.put_bytes(name, &bytes).map_err(worker::Error::from)?;

        builder.execute().await.map_err(worker::Error::from)
    }
}

impl<K, G> RateStore for WorkerStore<K, G>
where
    K: Hash + Eq + Send + Sync + Clone + 'static + AsRef<str>,
    G: RateGuard + Serialize + DeserializeOwned,
{
    type Error = worker::Error;
    type Key = K;
    type Guard = G;

    async fn load_guard<Q>(&self, depot: &Depot, key: &Q, refer: &Self::Guard) -> Result<Self::Guard, Self::Error>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq + Sync + AsRef<str>,
    {
        let guard = self.get(depot, key).await;
        if let Some(guard) = guard {
            Ok(guard)
        } else {
            Ok(refer.clone())
        }
    }

    async fn save_guard(&self, depot: &Depot, key: Self::Key, guard: Self::Guard) -> Result<(), Self::Error> {
        self.set(depot, key, guard).await
    }
}
