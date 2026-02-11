use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use anyhow::Result;

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(brokers: &str, group_id: &str, topics: &[&str]) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(topics)?;

        Ok(Self { consumer })
    }

    pub async fn consume<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(String, String) -> Result<()>,
    {
        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    let key = message
                        .key()
                        .and_then(|k| std::str::from_utf8(k).ok())
                        .unwrap_or("")
                        .to_string();

                    let payload = message
                        .payload()
                        .and_then(|p| std::str::from_utf8(p).ok())
                        .unwrap_or("")
                        .to_string();

                    if let Err(e) = handler(key, payload) {
                        tracing::error!("Error handling message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Kafka error: {}", e);
                }
            }
        }
    }
}
