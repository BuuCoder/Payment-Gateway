use actix::Addr;
use futures::StreamExt;
use redis::aio::MultiplexedConnection;
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::websocket::{BroadcastToRoomLocal, ChatServer, WsResponse};

pub struct RedisListener {
    redis_conn: MultiplexedConnection,
    chat_server: Addr<ChatServer>,
    subscribed_rooms: HashSet<String>,
}

impl RedisListener {
    pub fn new(redis_conn: MultiplexedConnection, chat_server: Addr<ChatServer>) -> Self {
        Self {
            redis_conn,
            chat_server,
            subscribed_rooms: HashSet::new(),
        }
    }

    pub async fn start(mut self) {
        info!("Starting Redis listener for chat synchronization");

        loop {
            match self.listen().await {
                Ok(_) => {
                    warn!("Redis listener stopped, restarting in 5 seconds...");
                }
                Err(e) => {
                    error!("Redis listener error: {}, restarting in 5 seconds...", e);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get Redis client from connection - use async connection for pub/sub
        let client = redis::Client::open("redis://redis:6379")?;
        let conn = client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        
        pubsub.psubscribe("chat:room:*").await?;

        info!("Subscribed to Redis pattern: chat:room:*");

        let mut pubsub_stream = pubsub.on_message();

        while let Some(msg) = pubsub_stream.next().await {
            let channel: String = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload()?;

            // Extract room_id from channel name (chat:room:{room_id})
            if let Some(room_id) = channel.strip_prefix("chat:room:") {
                self.handle_message(room_id, &payload).await;
            }
        }

        Ok(())
    }

    async fn handle_message(&self, room_id: &str, payload: &str) {
        match serde_json::from_str::<WsResponse>(payload) {
            Ok(message) => {
                // Broadcast ONLY to local connections (do not re-publish to Redis)
                self.chat_server.do_send(BroadcastToRoomLocal {
                    room_id: room_id.to_string(),
                    message,
                    exclude_user: None,
                });
            }
            Err(e) => {
                error!("Failed to parse Redis message: {}", e);
            }
        }
    }
}
