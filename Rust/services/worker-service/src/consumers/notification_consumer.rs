use messaging::kafka_consumer::KafkaConsumer;
use messaging::events::PaymentCreatedEvent;
use anyhow::Result;

pub async fn start(brokers: &str) -> Result<()> {
    tracing::info!("🔔 Notification consumer starting...");
    
    let consumer = KafkaConsumer::new(
        brokers,
        "notification-group",
        &["payment-events"]
    )?;
    
    consumer.consume(|key, payload| {
        tracing::info!("Notification consumer received message - Key: {}", key);
        
        match serde_json::from_str::<PaymentCreatedEvent>(&payload) {
            Ok(event) => {
                // Simulate sending notification
                tracing::info!(
                    "🔔 Sending notification for payment {} to user {}",
                    event.payment_id,
                    event.user_id
                );
                tracing::info!("   Amount: ${}, Status: {}", event.amount, event.status);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to parse event: {}", e);
                Err(anyhow::anyhow!("Parse error: {}", e))
            }
        }
    }).await
}
