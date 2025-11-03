use crate::cache::Cache;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub cache: Cache,
}

impl AppState {
    pub async fn new(db_pool: PgPool, cache_capacity: u64) -> Self {
        Self {
            db_pool,
            cache: Cache::new(cache_capacity),
        }
    }
}
