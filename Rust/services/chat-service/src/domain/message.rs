use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub room_id: String,
    pub sender_id: i64,
    pub content: String,
    pub message_type: String, // text, image, file, system
    pub metadata: Option<String>, // JSON for file info, etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub room_id: String,
    pub content: String,
    pub message_type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub room_id: String,
    pub sender_id: i64,
    pub sender_name: Option<String>,
    pub content: String,
    pub message_type: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(room_id: String, sender_id: i64, content: String, message_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            room_id,
            sender_id,
            content,
            message_type,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> MessageResponse {
        MessageResponse {
            id: self.id.clone(),
            room_id: self.room_id.clone(),
            sender_id: self.sender_id,
            sender_name: None,
            content: self.content.clone(),
            message_type: self.message_type.clone(),
            metadata: self.metadata.as_ref().and_then(|m| serde_json::from_str(m).ok()),
            created_at: self.created_at,
        }
    }
}
