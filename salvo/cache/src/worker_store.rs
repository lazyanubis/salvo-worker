//! Worker KV store module.
use std::borrow::Borrow;
use std::hash::Hash;
use std::time::Duration;

use salvo_core::http::{HeaderMap, StatusCode};
use salvo_core::{Depot, http};
use serde::{Deserialize, Serialize};

use crate::CachedBody;

use super::{CacheStore, CachedEntry};

/// A builder for [`MokaStore`].
pub struct Builder {
    key: String,
    live: Option<Duration>,
}
impl Builder {
    /// 距离插入时间
    /// ! 至少 60s
    pub fn time_to_live(mut self, duration: Duration) -> Self {
        self.live = Some(duration);
        self
    }

    /// 构建
    pub fn build(self) -> WorkerStore {
        WorkerStore {
            key: self.key,
            live: self.live,
        }
    }
}
/// A simple in-memory store for rate limiter.
pub struct WorkerStore {
    key: String,
    live: Option<Duration>,
}
impl WorkerStore {
    /// Create a new `MokaStore`.
    pub fn new(key: String) -> Self {
        Self { key, live: None }
    }

    /// Returns a [`Builder`], which can build a `MokaStore`.
    pub fn builder(key: String) -> Builder {
        Builder { key, live: None }
    }
}

impl CacheStore for WorkerStore {
    type Error = worker::Error;
    type Key = String;

    #[worker::send]
    async fn load_entry<Q>(&self, depot: &Depot, key: &Q) -> Option<CachedEntry>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq + Sync + AsRef<str>,
    {
        let env = depot.obtain::<worker::Env>().ok()?;
        let kv = env.kv(&self.key).ok()?;

        let name = key.as_ref();
        let builder = kv.get(name);
        let bytes = builder.bytes().await.ok()??;

        let entry: InnerCachedEntry = ciborium::de::from_reader(&bytes[..]).ok()?;
        let entry: CachedEntry = entry.try_into().ok()?;

        Some(entry)
    }

    #[worker::send]
    async fn save_entry(&self, depot: &Depot, key: Self::Key, entry: CachedEntry) -> Result<(), Self::Error> {
        let env = depot
            .obtain::<worker::Env>()
            .map_err(|_| worker::Error::Json(("obtain Env failed".to_string(), 1)))?;
        let kv = env.kv(&self.key)?;

        let name = key.as_ref();
        let entry: InnerCachedEntry = entry.try_into()?;
        let mut bytes = vec![];
        ciborium::ser::into_writer(&entry, &mut bytes).map_err(|err| worker::Error::Json((format!("{err:?}"), 1)))?;

        let mut builder = kv.put_bytes(name, &bytes).map_err(|err| worker::Error::from(err))?;
        if let Some(live) = self.live {
            builder = builder.expiration_ttl(live.as_secs());
        }

        builder.execute().await.map_err(|err| worker::Error::from(err))
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum InnerCachedBody {
    None,
    Once(Vec<u8>),
    Chunks(Vec<Vec<u8>>),
}

#[derive(Debug, Serialize, Deserialize)]
struct InnerCachedEntry {
    status: Option<u16>,
    headers: Vec<(String, String)>,
    body: InnerCachedBody,
}

impl TryFrom<CachedEntry> for InnerCachedEntry {
    type Error = worker::Error;

    fn try_from(entry: CachedEntry) -> Result<Self, Self::Error> {
        let status = entry.status.map(|s| s.as_u16());
        let headers = entry
            .headers
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.to_string(),
                    v.to_str()
                        .map_err(|err| worker::Error::Json((format!("can not get header value: {err:?}"), 1)))?
                        .to_string(),
                ))
            })
            .collect::<Result<Vec<(String, String)>, Self::Error>>()?;
        let body = match entry.body {
            CachedBody::None => InnerCachedBody::None,
            CachedBody::Once(bytes) => InnerCachedBody::Once(bytes.into()),
            CachedBody::Chunks(chunks) => InnerCachedBody::Chunks(chunks.into_iter().map(|b| b.into()).collect()),
        };
        Ok(Self { status, headers, body })
    }
}

impl TryFrom<InnerCachedEntry> for CachedEntry {
    type Error = worker::Error;

    fn try_from(entry: InnerCachedEntry) -> Result<Self, Self::Error> {
        let status = entry.status.map(|s| StatusCode::from_u16(s)).transpose()?;
        let mut headers = HeaderMap::with_capacity(entry.headers.len());
        for (name, value) in entry.headers {
            headers.insert(http::HeaderName::from_bytes(name.as_bytes())?, value.parse()?);
        }
        let body = match entry.body {
            InnerCachedBody::None => CachedBody::None,
            InnerCachedBody::Once(bytes) => CachedBody::Once(bytes.into()),
            InnerCachedBody::Chunks(chunks) => CachedBody::Chunks(chunks.into_iter().map(|b| b.into()).collect()),
        };
        Ok(Self { status, headers, body })
    }
}
