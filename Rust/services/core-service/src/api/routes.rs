use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health_check))
        .service(
            web::scope("/api/v1")
                .service(
                    web::scope("/users")
                        .route("", web::get().to(handlers::get_users))
                        .route("/{id}", web::get().to(handlers::get_user))
                )
        );
}
