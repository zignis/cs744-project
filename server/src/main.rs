use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use num_cpus;
use server::routes;
use server::state::AppState;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing_subscriber;

const DEFAULT_POOL_SIZE: u32 = 16;
const DEFAULT_CACHE_SIZE: u64 = 128_000;

async fn not_found() -> impl Responder {
    HttpResponse::NotFound()
        .content_type(ContentType::plaintext())
        .body("not found")
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let bind = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:6464".into());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool_size: u32 = env::var("DB_POOL_SIZE")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(DEFAULT_POOL_SIZE);
    let cache_size: u64 = env::var("CACHE_SIZE")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(DEFAULT_CACHE_SIZE);

    let pool = PgPoolOptions::new()
        .max_connections(db_pool_size)
        .connect(&database_url)
        .await?;

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("ran database migrations"),
        Err(err) => {
            eprintln!("failed to run database migrations: {:?}", err);
            std::process::exit(1);
        }
    }

    let state = AppState::new(pool, cache_size).await;
    let data = web::Data::new(state);

    println!("starting at http://{}", bind);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(routes::init_routes)
            .default_service(web::route().to(not_found))
    })
    .workers(num_cpus::get()) // one worker per cpu core
    .bind(bind)?
    .run()
    .await?;

    Ok(())
}
