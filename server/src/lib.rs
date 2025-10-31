use sqlx::{Pool, Postgres};
mod config;

pub use config::*;

/// The application state.
pub struct AppState {
    /// The environment configuration.
    pub config: config::Config,
    /// Postgres connection pool
    pub db_pool: Pool<Postgres>,
}
