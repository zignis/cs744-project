use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct KVPair {
    pub key: String,
    pub value: String,
}

#[derive(Clone)]
pub struct Cache {
    inner: Arc<DashMap<String, String>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).map(|x| x.clone())
    }

    pub async fn insert(&self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    pub async fn remove(&self, key: &str) {
        self.inner.remove(key);
    }

    pub async fn len(&self) -> usize {
        self.inner.len()
    }
}
