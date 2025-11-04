use crate::error::AppError;
use crate::state::AppState;
use actix_web::{delete, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fragments {
    key: String,
}

#[delete("/{key}")]
async fn get_kv(
    path: web::Path<Fragments>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let key = path.key.clone();
    match sqlx::query!("DELETE FROM kv_store WHERE key = $1", key)
        .execute(&data.db_pool)
        .await?
        .rows_affected()
    {
        0 => Err(AppError::NotFound(key)),
        _ => {
            data.cache.remove(&key).await;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_kv);
}
