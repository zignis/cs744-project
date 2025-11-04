use crate::error::AppError;
use crate::state::AppState;
use actix_web::{get, web, HttpResponse};

#[get("/stats")]
async fn get_stats(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(data.cache.stats()))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}
