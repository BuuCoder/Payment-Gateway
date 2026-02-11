use actix_web::web;
use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health_check))
        .service(
            web::scope("/api")
                .route("/payments", web::post().to(handlers::create_payment))
                .route("/payment_intents/{intent_id}", web::get().to(handlers::retrieve_payment))
        )
        .route("/webhooks/stripe", web::post().to(handlers::stripe_webhook));
}
