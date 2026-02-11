mod consumers;

use std::env;
use tokio::task;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let kafka_brokers = env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());
    
    tracing::info!("🔧 Worker Service starting...");
    tracing::info!("Kafka brokers: {}", kafka_brokers);
    
    // Spawn email consumer
    let email_brokers = kafka_brokers.clone();
    let email_task = task::spawn(async move {
        consumers::email_consumer::start(&email_brokers).await
    });
    
    // Spawn notification consumer
    let notif_brokers = kafka_brokers.clone();
    let notif_task = task::spawn(async move {
        consumers::notification_consumer::start(&notif_brokers).await
    });
    
    // Wait for both consumers
    let _ = tokio::try_join!(email_task, notif_task)?;
    
    Ok(())
}
