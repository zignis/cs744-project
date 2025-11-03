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
    sqlx::query!(
        r#"
INSERT INTO kv_store (key, value)
VALUES ($1, $2)
ON CONFLICT (key)
    DO UPDATE SET value = EXCLUDED.value
        "#,
        payload.key,
        payload.value
    )
    .execute(&data.db_pool)
    .await?;

    data.cache
        .insert(payload.key.clone(), payload.value.clone())
        .await;

    Ok(HttpResponse::Created().finish())
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(post_kv);
}
