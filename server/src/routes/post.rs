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

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_app::setup_test_app;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn can_insert_key(pool: PgPool) -> sqlx::Result<()> {
        let app = setup_test_app(pool.clone()).await;

        // create
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({"key": "key_1", "value": "value_1"}))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::CREATED);

        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_and_read_body(&app, req).await;
        assert_eq!(str::from_utf8(&res).unwrap(), "value_1");

        // update
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({"key": "key_1", "value": "value_2"}))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_and_read_body(&app, req).await;
        assert_eq!(str::from_utf8(&res).unwrap(), "value_2");

        Ok(())
    }
}
