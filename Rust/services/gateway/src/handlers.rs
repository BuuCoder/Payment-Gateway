use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use messaging::kafka_producer::KafkaProducer;
use messaging::events::PaymentCreatedEvent;
use chrono::Utc;
use crate::clients::StripeClient;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub user_id: i32,
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
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        service: "gateway".to_string(),
    })
}

pub async fn create_payment(
    pool: web::Data<MySqlPool>,
    stripe_client: web::Data<StripeClient>,
    producer: web::Data<KafkaProducer>,
    request: web::Json<CreatePaymentRequest>,
) -> impl Responder {
    let currency = request.currency.clone().unwrap_or_else(|| "USD".to_string());
    let payment_method = request.payment_method.clone().unwrap_or_else(|| "card".to_string());
    let amount_cents = (request.amount * 100.0) as i64;

    // Create payment intent with Stripe
    let payment_intent = match stripe_client.create_payment_intent(amount_cents, &currency).await {
        Ok(intent) => intent,
        Err(e) => {
            tracing::error!("Stripe API error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create payment intent: {}", e)
            }));
        }
    };

    // Insert payment into database
    let result = sqlx::query(
        "INSERT INTO payments (user_id, amount, currency, status, payment_method, stripe_payment_intent_id, stripe_client_secret) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(request.user_id)
    .bind(request.amount)
    .bind(&currency)
    .bind("pending")
    .bind(&payment_method)
    .bind(&payment_intent.id)
    .bind(&payment_intent.client_secret)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let payment_id = result.last_insert_id() as i32;

            // Send event to Kafka
            let event = PaymentCreatedEvent {
                payment_id,
                user_id: request.user_id,
                amount: request.amount,
                status: "pending".to_string(),
                timestamp: Utc::now().to_rfc3339(),
            };

            let payload = serde_json::to_string(&event).unwrap();
            if let Err(e) = producer.send_message("payment-events", &payment_id.to_string(), &payload).await {
                tracing::error!("Failed to send Kafka message: {}", e);
            }

            HttpResponse::Created().json(CreatePaymentResponse {
                id: payment_id,
                user_id: request.user_id,
                amount: request.amount,
                currency,
                status: "pending".to_string(),
                client_secret: payment_intent.client_secret,
                stripe_payment_intent_id: payment_intent.id,
            })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create payment: {}", e)
            }))
        }
    }
}

pub async fn retrieve_payment(
    pool: web::Data<MySqlPool>,
    stripe_client: web::Data<StripeClient>,
    intent_id: web::Path<String>,
) -> impl Responder {
    // Get payment intent from Stripe
    let payment_intent = match stripe_client.retrieve_payment_intent(&intent_id).await {
        Ok(intent) => intent,
        Err(e) => {
            tracing::error!("Stripe API error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve payment intent: {}", e)
            }));
        }
    };

    // Update payment status in database
    let update_result = sqlx::query(
        "UPDATE payments SET status = ? WHERE stripe_payment_intent_id = ?"
    )
    .bind(&payment_intent.status)
    .bind(&intent_id.as_str())
    .execute(pool.get_ref())
    .await;

    if let Err(e) = update_result {
        tracing::error!("Failed to update payment status: {}", e);
    }

    // Get payment from database
    let payment = sqlx::query_as::<_, (i32, i32, f64, String, String, String)>(
        "SELECT id, user_id, amount, currency, status, stripe_payment_intent_id FROM payments WHERE stripe_payment_intent_id = ?"
    )
    .bind(&intent_id.as_str())
    .fetch_one(pool.get_ref())
    .await;

    match payment {
        Ok((id, user_id, amount, currency, status, stripe_id)) => {
            HttpResponse::Ok().json(PaymentStatusResponse {
                id,
                user_id,
                amount,
                currency,
                status,
                stripe_payment_intent_id: stripe_id,
            })
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
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
    pool: web::Data<MySqlPool>,
    _producer: web::Data<KafkaProducer>,
    _req: HttpRequest,
    payload: web::Json<StripeWebhook>,
) -> impl Responder {
    tracing::info!("Received Stripe webhook: {}", payload.event_type);

    match payload.event_type.as_str() {
        "payment_intent.succeeded" => {
            if let Some(intent_id) = payload.data.get("object").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                // Update payment status
                let result = sqlx::query(
                    "UPDATE payments SET status = 'succeeded' WHERE stripe_payment_intent_id = ?"
                )
                .bind(intent_id)
                .execute(pool.get_ref())
                .await;

                if let Err(e) = result {
                    tracing::error!("Failed to update payment: {}", e);
                }

                tracing::info!("Payment succeeded: {}", intent_id);
            }
        }
        "payment_intent.payment_failed" => {
            if let Some(intent_id) = payload.data.get("object").and_then(|o| o.get("id")).and_then(|id| id.as_str()) {
                let result = sqlx::query(
                    "UPDATE payments SET status = 'failed' WHERE stripe_payment_intent_id = ?"
                )
                .bind(intent_id)
                .execute(pool.get_ref())
                .await;

                if let Err(e) = result {
                    tracing::error!("Failed to update payment: {}", e);
                }

                tracing::info!("Payment failed: {}", intent_id);
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
