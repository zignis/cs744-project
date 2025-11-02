use actix_web::web;

mod delete;
mod get;
mod post;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    get::init_routes(cfg);
    post::init_routes(cfg);
    delete::init_routes(cfg);
}
