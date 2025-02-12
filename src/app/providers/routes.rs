use actix_web::{web};

pub fn register(cfg: &mut web::ServiceConfig) {
    crate::routes::api::register(cfg);
    crate::routes::css::register(cfg);
    crate::routes::js::register(cfg);
    crate::routes::web::register(cfg);
}