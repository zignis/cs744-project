use crate::routes;
use crate::state::AppState;
use sqlx::PgPool;

pub async fn setup_test_app(
    pool: PgPool,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let state = AppState::new(pool, 64).await;
    actix_web::test::init_service(
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(state))
            .configure(routes::init_routes),
    )
    .await
}
