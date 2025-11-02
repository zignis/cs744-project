use crate::cache::Cache;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub cache: Arc<Cache>,
}

impl AppState {
    pub async fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            cache: Arc::new(Cache::new()),
        }
    }
}
