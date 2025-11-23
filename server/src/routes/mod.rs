use actix_web::web;

mod delete;
mod flush;
mod get;
mod post;
mod stats;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    stats::init_routes(cfg);
    get::init_routes(cfg);
    post::init_routes(cfg);
    delete::init_routes(cfg);
    flush::init_routes(cfg);
}
