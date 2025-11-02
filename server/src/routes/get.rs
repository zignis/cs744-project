use crate::error::AppError;
use crate::state::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::Row;

#[derive(Deserialize)]
struct Fragments {
    key: String,
}

#[get("/{key}")]
async fn get_kv(
    path: web::Path<Fragments>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let key = path.key.clone();

    if let Some(value) = data.cache.as_ref().get(&key).await {
        return Ok(HttpResponse::Ok().body(value));
    }

    let row = sqlx::query("SELECT value FROM kv_store WHERE key = $1")
        .bind(key.clone())
        .fetch_one(&data.db_pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => AppError::NotFound(key.clone()),
            other => AppError::Database(other),
        })?;
    let value = row.get::<String, _>("value");

    data.cache.as_ref().insert(key, value.clone()).await;

    Ok(HttpResponse::Ok().body(value))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_kv);
}
