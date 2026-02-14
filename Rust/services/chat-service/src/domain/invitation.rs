use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoomInvitation {
    pub id: i64,
    pub room_id: String,
    pub user_id: i64,
    pub invited_by: i64,
    pub status: String, // pending, accepted, declined
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationResponse {
    pub id: i64,
    pub room_id: String,
    pub room_name: Option<String>,
    pub room_type: String,
    pub invited_by: i64,
    pub invited_by_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptInvitationResponse {
    pub room: crate::domain::RoomResponse,
}
