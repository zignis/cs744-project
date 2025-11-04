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

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_app::setup_test_app;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn can_remove_key(pool: PgPool) -> sqlx::Result<()> {
        let app = setup_test_app(pool.clone()).await;

        // insert key
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({"key": "key_1", "value": "value_1"}))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        // delete
        let req = test::TestRequest::delete().uri("/key_1").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
