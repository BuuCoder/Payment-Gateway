use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/health", web::get().to(handlers::health_check))
            .route("/login", web::post().to(handlers::login))
            .route("/register", web::post().to(handlers::register))
    );
}
