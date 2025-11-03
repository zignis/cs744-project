use moka::policy::EvictionPolicy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct KVPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Cache {
    map: moka::future::Cache<String, String>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
}

impl Cache {
    pub fn new(capacity: u64) -> Self {
        Self {
            map: moka::future::Cache::builder()
                .max_capacity(capacity)
                .eviction_policy(EvictionPolicy::lru())
                .build(),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        match self.map.get(key).await {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub async fn insert(&self, key: String, value: String) {
        self.map.insert(key, value).await;
    }

    pub async fn remove(&self, key: &str) {
        self.map.invalidate(key).await;
    }

    pub async fn len(&self) -> u64 {
        self.map.entry_count()
    }

    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }
}
