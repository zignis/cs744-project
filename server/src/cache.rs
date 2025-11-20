use moka::policy::EvictionPolicy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct KVPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Cache {
    capacity: u64,
    map: moka::future::Cache<String, String>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub capacity: u64,
    pub entries: u64,
    pub hits: u64,
    pub misses: u64,
}

impl Cache {
    pub fn new(capacity: u64) -> Self {
        Self {
            capacity,
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

    pub fn len(&self) -> u64 {
        self.map.entry_count()
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            capacity: self.capacity,
            entries: self.map.entry_count(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
        }
    }

    pub fn flush(&self) {
        self.map.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;

    #[actix_web::test]
    async fn cache_hit_and_miss_counts() {
        let cache = Cache::new(8);

        assert_eq!(cache.get("invalid").await, None);
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 1);

        cache.insert("key_1".into(), "value_1".into()).await;
        assert_eq!(cache.get("key_1").await, Some("value_1".to_string()));
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);

        cache.remove("key_1").await;
        assert_eq!(cache.get("key_1").await, None);
    }
}
