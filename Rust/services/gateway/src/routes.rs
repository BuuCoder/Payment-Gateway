use actix_web::web;
use crate::handlers;
use authz::AuthMiddleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

    cfg
        // Public routes (no auth required)
        .route("/health", web::get().to(handlers::health_check))
        .route("/webhooks/stripe", web::post().to(handlers::stripe_webhook))
        // Protected routes (auth required)
        .service(
            web::scope("/api/v1")
                .wrap(AuthMiddleware::new(jwt_secret))
                .route("/payments", web::post().to(handlers::create_payment))
                .route("/payment_intents/{intent_id}", web::get().to(handlers::retrieve_payment))
        );
}
