use anyhow::{Result, anyhow};
use messaging::kafka_producer::KafkaProducer;
use messaging::events::PaymentCreatedEvent;
use chrono::Utc;

use crate::domain::{Payment, PaymentStatus};
use crate::repo::PaymentRepository;
use crate::clients::StripeClient;

#[derive(Clone)]
pub struct PaymentService {
    payment_repo: PaymentRepository,
    stripe_client: StripeClient,
    kafka_producer: KafkaProducer,
}

impl PaymentService {
    pub fn new(
        payment_repo: PaymentRepository,
        stripe_client: StripeClient,
        kafka_producer: KafkaProducer,
    ) -> Self {
        Self {
            payment_repo,
            stripe_client,
            kafka_producer,
        }
    }

    pub async fn create_payment(
        &self,
        user_id: i32,
        amount: f64,
        currency: &str,
        payment_method: &str,
    ) -> Result<(i32, String, String)> {
        // Calculate amount in cents for Stripe
        let amount_cents = (amount * 100.0) as i64;

        // Create payment intent with Stripe
        let payment_intent = self.stripe_client
            .create_payment_intent(amount_cents, currency)
            .await
            .map_err(|e| anyhow!("Stripe API error: {}", e))?;

        // Save payment to database
        let payment_id = self.payment_repo
            .create(
                user_id,
                amount,
                currency,
                PaymentStatus::Pending.as_str(),
                payment_method,
                &payment_intent.id,
                &payment_intent.client_secret,
            )
            .await?;

        // Publish event to Kafka
        let event = PaymentCreatedEvent {
            payment_id,
            user_id,
            amount,
            status: PaymentStatus::Pending.as_str().to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_string(&event)?;
        if let Err(e) = self.kafka_producer
            .send_message("payment-events", &payment_id.to_string(), &payload)
            .await
        {
            tracing::error!("Failed to send Kafka message: {}", e);
        }

        Ok((payment_id, payment_intent.client_secret, payment_intent.id))
    }

    pub async fn retrieve_payment(&self, intent_id: &str) -> Result<Payment> {
        // Get payment intent from Stripe
        let payment_intent = self.stripe_client
            .retrieve_payment_intent(intent_id)
            .await
            .map_err(|e| anyhow!("Stripe API error: {}", e))?;

        // Update payment status in database
        if let Err(e) = self.payment_repo
            .update_status(intent_id, &payment_intent.status)
            .await
        {
            tracing::error!("Failed to update payment status: {}", e);
        }

        // Get payment from database
        let payment = self.payment_repo
            .find_by_stripe_intent_id(intent_id)
            .await?
            .ok_or_else(|| anyhow!("Payment not found"))?;

        Ok(payment)
    }

    pub async fn update_payment_status(&self, intent_id: &str, status: &str) -> Result<()> {
        self.payment_repo.update_status(intent_id, status).await?;
        tracing::info!("Payment status updated: {} -> {}", intent_id, status);
        Ok(())
    }
}
