use crate::error::AppError;
use crate::state::AppState;
use actix_web::{post, web, HttpResponse};
use actix_web_validator::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct Request {
    #[validate(length(min = 1, max = 512, message = "invalid key length"))]
    key: String,
    #[validate(length(min = 1, max = 4096, message = "invalid value length"))]
    value: String,
}

#[post("/")]
async fn post_kv(
    payload: Json<Request>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let row = sqlx::query!(
        r#"
INSERT INTO kv_store (key, value)
VALUES ($1, $2)
ON CONFLICT (key)
DO UPDATE
SET value      = EXCLUDED.value,
    updated_at = NOW()
RETURNING (created_at = updated_at) AS inserted
        "#,
        payload.key,
        payload.value
    )
    .fetch_one(&data.db_pool)
    .await?;

    data.cache
        .insert(payload.key.clone(), payload.value.clone())
        .await;

    Ok(if row.inserted.unwrap_or(false) {
        HttpResponse::Created()
    } else {
        HttpResponse::NoContent()
    }
    .finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(post_kv);
}
