use crate::error::AppError;
use crate::state::AppState;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

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

    if let Some(value) = data.cache.get(&key).await {
        return Ok(HttpResponse::Ok().body(value));
    }

    let value = sqlx::query!("SELECT value FROM kv_store WHERE key = $1", key)
        .fetch_one(&data.db_pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => AppError::NotFound(key.clone()),
            other => AppError::Database(other),
        })?
        .value;

    data.cache.insert(key, value.clone()).await;

    Ok(HttpResponse::Ok().body(value))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_kv);
}

#[cfg(test)]
mod tests {
    use crate::test_utils::setup_app::setup_test_app;
    use actix_web::test;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn can_get_key(pool: PgPool) -> sqlx::Result<()> {
        let app = setup_test_app(pool.clone()).await;

        sqlx::query!(
            "INSERT INTO kv_store (key, value) VALUES ($1, $2)",
            "key_1",
            "value_1"
        )
        .execute(&pool)
        .await?;

        // get key from database (will populate it into the cache)
        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_and_read_body(&app, req).await;
        assert_eq!(str::from_utf8(&res).unwrap(), "value_1");

        // get key from cache
        let req = test::TestRequest::get().uri("/key_1").to_request();
        let res = test::call_and_read_body(&app, req).await;
        assert_eq!(str::from_utf8(&res).unwrap(), "value_1");

        Ok(())
    }
}
