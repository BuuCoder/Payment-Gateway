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
    #[serde(rename = "room_created")]
    RoomCreated {
        room_id: String,
        room_name: Option<String>,
        room_type: String,
    },
    #[serde(rename = "invitation_received")]
    InvitationReceived {
        invitation_id: i64,
        room_id: String,
        room_name: Option<String>,
        invited_by: i64,
        invited_by_name: String,
    },
    #[serde(rename = "member_joined")]
    MemberJoined {
        room_id: String,
        user_id: i64,
        user_name: String,
    },
    #[serde(rename = "member_left")]
    MemberLeft {
        room_id: String,
        user_id: i64,
        user_name: String,
    },
    #[serde(rename = "room_updated")]
    RoomUpdated {
        room_id: String,
        last_message_at: String,
    },
    #[serde(rename = "unread_updated")]
    UnreadUpdated {
        room_id: String,
        unread_count: i64,
    },
    #[serde(rename = "user_online")]
    UserOnline {
        user_id: i64,
        user_name: String,
    },
    #[serde(rename = "user_offline")]
    UserOffline {
        user_id: i64,
    },
    #[serde(rename = "room_presence")]
    RoomPresence {
        room_id: String,
        online_users: Vec<i64>,
    },
    #[serde(rename = "connection_replaced")]
    ConnectionReplaced {
        message: String,
    },
    #[serde(rename = "rate_limit_exceeded")]
    RateLimitExceeded {
        event_type: String,
        retry_after: f64,
        message: String,
    },
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
    pub session_id: String,
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

// Broadcast to specific users
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastToUsers {
    pub user_ids: Vec<i64>,
    pub message: WsResponse,
}

// Track user typing
#[derive(Message)]
#[rtype(result = "()")]
pub struct UserTyping {
    pub user_id: i64,
    pub room_id: String,
}

// Check rate limit
#[derive(Message)]
#[rtype(result = "Result<(), f64>")]
pub struct CheckRateLimit {
    pub user_id: i64,
    pub event_type: String, // "message", "typing", "room_action"
}
