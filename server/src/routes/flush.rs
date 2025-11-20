use crate::error::AppError;
use crate::state::AppState;
use actix_web::{post, web, HttpResponse};

#[post("/flush")]
async fn flush_kv(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let result = sqlx::query!(r#"DELETE FROM kv_store"#)
        .execute(&data.db_pool)
        .await?;

    data.cache.flush();

    Ok(HttpResponse::Ok().body(format!("flushed {} pairs", result.rows_affected())))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(flush_kv);
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_app::setup_test_app;
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn can_flush(pool: PgPool) -> sqlx::Result<()> {
        let app = setup_test_app(pool.clone()).await;

        // create
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&json!({"key": "key_1", "value": "value_1"}))
            .to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_and_read_body(&app, req).await;
        assert_eq!(str::from_utf8(&res).unwrap(), "value_1");

        // flush
        let req = test::TestRequest::post().uri("/flush").to_request();
        let res = test::call_service(&app, req).await;
        assert!(res.status().is_success());

        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
