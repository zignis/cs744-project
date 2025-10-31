use crate::AppState;
use crate::get_app_config;
use axum::{Router, extract::State, routing::get};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
mod store;

fn make_api() -> Router<AppState> {
    Router::new().route("/get_val", get(posts_handler))
}

async fn posts_handler(State(state): State<AppState>) {
    "hello"
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    match get_app_config() {
        Ok(config) => {
            let host = config.host.to_string();
            let port = config.port.clone().parse::<u16>().unwrap();

            println!("starting server at http://{}:{}", &host, &port);

            // Postgres
            let db_pool = match PgPoolOptions::new()
                .max_connections(30)
                .min_connections(1)
                .idle_timeout(Some(Duration::from_secs(120)))
                .connect(&config.database_url)
                .await
            {
                Ok(pool) => {
                    println!("connected to postgres");

                    // Run migrations.
                    match sqlx::migrate!("./migrations").run(&pool).await {
                        Ok(_) => {
                            println!("Successfully ran database migrations");
                        }
                        Err(err) => {
                            eprintln!("failed to run database migrations: {:?}", err);
                            std::process::exit(1);
                        }
                    }

                    pool
                }
                Err(err) => {
                    eprintln!("failed to connect to Postgres: {:?}", err);
                    std::process::exit(1);
                }
            };

            let app = Router::new().nest("/api", make_api()).with_state(AppState {
                config: get_app_config().unwrap(),
                db_pool,
            });
            let listener = tokio::net::TcpListener::bind(&format!("{host}:{port}"))
                .await
                .unwrap();
            axum::serve(listener, app).await.unwrap();

            Ok(())
        }
        Err(error) => {
            eprintln!("Environment configuration error: {:#?}", error);
            Ok(())
        }
    }
}
