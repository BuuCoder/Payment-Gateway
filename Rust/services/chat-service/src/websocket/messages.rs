use actix::prelude::*;
use serde::{Deserialize, Serialize};

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "message")]
    Message {
        room_id: String,
        content: String,
        message_type: String,
        metadata: Option<serde_json::Value>,
    },
    #[serde(rename = "join_room")]
    JoinRoom { room_id: String },
    #[serde(rename = "leave_room")]
    LeaveRoom { room_id: String },
    #[serde(rename = "typing")]
    Typing { room_id: String, is_typing: bool },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsResponse {
    #[serde(rename = "message")]
    Message {
        id: String,
        room_id: String,
        sender_id: i64,
        sender_name: Option<String>,
        content: String,
        message_type: String,
        metadata: Option<serde_json::Value>,
        created_at: String,
    },
    #[serde(rename = "typing")]
    Typing {
        room_id: String,
        user_id: i64,
        user_name: Option<String>,
        is_typing: bool,
    },
    #[serde(rename = "joined")]
    Joined { room_id: String },
    #[serde(rename = "left")]
    Left { room_id: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "pong")]
    Pong,
}

// Actor messages for ChatServer
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsResponseMessage>,
    pub user_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub user_id: i64,
    pub room_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoom {
    pub user_id: i64,
    pub room_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub user_id: i64,
    pub room_id: String,
    pub message: WsResponse,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastToRoom {
    pub room_id: String,
    pub message: WsResponse,
    pub exclude_user: Option<i64>,
}

// Broadcast only to local connections (used by Redis listener to avoid loops)
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastToRoomLocal {
    pub room_id: String,
    pub message: WsResponse,
    pub exclude_user: Option<i64>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsResponseMessage(pub WsResponse);

// Get connection count for health check
#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetConnectionCount;
