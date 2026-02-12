use actix_web::web;
use super::handlers;
use authz::AuthMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

    cfg
        // Public route
        .route("/health", web::get().to(handlers::health_check))
        // Protected routes
        .service(
            web::scope("/api/v1")
                .wrap(AuthMiddleware::new(jwt_secret))
                .service(
                    web::scope("/users")
                        .route("", web::get().to(handlers::get_users))
                        .route("/{id}", web::get().to(handlers::get_user))
                )
        );
}
