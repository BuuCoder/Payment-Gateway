use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health_check))
        .service(
            web::scope("/api/v1/auth")
                .route("/login", web::post().to(handlers::login))
                .route("/register", web::post().to(handlers::register))
        );
}
