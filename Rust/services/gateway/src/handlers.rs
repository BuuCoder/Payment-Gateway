use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use authz::Claims;
use crate::service::PaymentService;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    instance: String,
}

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: f64,
    pub currency: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Serialize)]
pub struct CreatePaymentResponse {
    pub id: i32,
    pub user_id: i32,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub client_secret: String,
    pub stripe_payment_intent_id: String,
}

#[derive(Serialize)]
pub struct PaymentStatusResponse {
    pub id: i32,
    pub user_id: i32,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub stripe_payment_intent_id: String,
}

pub async fn health_check() -> impl Responder {
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
    
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        service: "gateway".to_string(),
        instance: hostname,
    })
}

pub async fn create_payment(
    claims: web::ReqData<Claims>,
    payment_service: web::Data<PaymentService>,
    request: web::Json<CreatePaymentRequest>,
) -> impl Responder {
    let user_id = claims.user_id;
    tracing::info!("Creating payment for user: {} ({})", claims.sub, user_id);
    
    let currency = request.currency.clone().unwrap_or_else(|| "USD".to_string());
    let payment_method = request.payment_method.clone().unwrap_or_else(|| "card".to_string());

    match payment_service.create_payment(user_id, request.amount, &currency, &payment_method).await {
        Ok((payment_id, client_secret, stripe_payment_intent_id)) => {
            HttpResponse::Created().json(CreatePaymentResponse {
                id: payment_id,
                user_id,
                amount: request.amount,
                currency,
                status: "pending".to_string(),
                client_secret,
                stripe_payment_intent_id,
            })
        }
        Err(e) => {
            tracing::error!("Payment creation error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create payment: {}", e)
            }))
        }
    }
}

pub async fn retrieve_payment(
    payment_service: web::Data<PaymentService>,
    intent_id: web::Path<String>,
) -> impl Responder {
    match payment_service.retrieve_payment(&intent_id).await {
        Ok(payment) => {
            HttpResponse::Ok().json(PaymentStatusResponse {
                id: payment.id,
                user_id: payment.user_id,
                amount: payment.amount,
                currency: payment.currency,
                status: payment.status,
                stripe_payment_intent_id: payment.stripe_payment_intent_id.unwrap_or_default(),
            })
        }
        Err(e) => {
            tracing::error!("Payment retrieval error: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Payment not found"
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct StripeWebhook {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

pub async fn stripe_webhook(
    payment_service: web::Data<PaymentService>,
    _req: HttpRequest,
    payload: web::Json<StripeWebhook>,
) -> impl Responder {
    tracing::info!("Received Stripe webhook: {}", payload.event_type);

    match payload.event_type.as_str() {
        "payment_intent.succeeded" => {
            if let Some(intent_id) = payload.data.get("object").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                if let Err(e) = payment_service.update_payment_status(intent_id, "succeeded").await {
                    tracing::error!("Failed to update payment: {}", e);
                }
            }
        }
        "payment_intent.payment_failed" => {
            if let Some(intent_id) = payload.data.get("object").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                if let Err(e) = payment_service.update_payment_status(intent_id, "failed").await {
                    tracing::error!("Failed to update payment: {}", e);
                }
            }
        }
        _ => {
            tracing::info!("Unhandled webhook event: {}", payload.event_type);
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "received": true
    }))
}
