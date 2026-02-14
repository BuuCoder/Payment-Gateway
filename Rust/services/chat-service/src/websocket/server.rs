use actix::prelude::*;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::messages::*;
use super::rate_limiter::{RateLimiter, EventType};

const MAX_CONNECTIONS_PER_USER: usize = 5;

#[derive(Clone)]
pub struct SessionInfo {
    pub addr: Recipient<WsResponseMessage>,
    pub connected_at: Instant,
    pub session_id: String,
}

pub struct ChatServer {
    // user_id -> list of sessions (ordered by connected_at, oldest first)
    sessions: HashMap<i64, Vec<SessionInfo>>,
    // room_id -> set of user_ids
    rooms: HashMap<String, HashSet<i64>>,
    // user_id -> set of room_ids (for presence tracking)
    user_rooms: HashMap<i64, HashSet<String>>,
    // room_id -> (user_id -> last_typed_time)
    typing_users: HashMap<String, HashMap<i64, Instant>>,
    // Redis connection for pub/sub
    redis_conn: MultiplexedConnection,
    // Rate limiter
    rate_limiter: RateLimiter,
}

impl ChatServer {
    pub fn new(redis_conn: MultiplexedConnection) -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            user_rooms: HashMap::new(),
            typing_users: HashMap::new(),
            redis_conn,
            rate_limiter: RateLimiter::new(),
        }
    }

    fn send_message_to_user(&self, user_id: i64, message: &WsResponse) {
        if let Some(sessions) = self.sessions.get(&user_id) {
            // Send to all sessions of this user
            for session in sessions {
                let _ = session.addr.do_send(WsResponseMessage(message.clone()));
            }
        }
    }
    
    pub fn check_rate_limit(&mut self, user_id: i64, event_type: EventType) -> Result<(), f64> {
        self.rate_limiter.check_rate_limit(user_id, event_type)
    }

    async fn publish_to_redis(&mut self, channel: &str, message: &WsResponse) {
        if let Ok(json) = serde_json::to_string(message) {
            let result: Result<(), redis::RedisError> =
                self.redis_conn.publish(channel, json).await;
            if let Err(e) = result {
                error!("Failed to publish to Redis: {}", e);
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("ChatServer started");
        
        // Cleanup stale typing indicators every 3 seconds
        ctx.run_interval(Duration::from_secs(3), |act, _ctx| {
            let now = Instant::now();
            let mut stopped_typing: Vec<(String, i64)> = Vec::new();
            
            for (room_id, typing_map) in &mut act.typing_users {
                let stale_users: Vec<i64> = typing_map
                    .iter()
                    .filter(|(_, &last_typed)| now.duration_since(last_typed) > Duration::from_secs(3))
                    .map(|(&uid, _)| uid)
                    .collect();
                
                for user_id in stale_users {
                    typing_map.remove(&user_id);
                    stopped_typing.push((room_id.clone(), user_id));
                }
            }
            
            // Broadcast stopped typing events
            for (room_id, user_id) in stopped_typing {
                if let Some(users) = act.rooms.get(&room_id) {
                    let stop_typing_msg = WsResponse::Typing {
                        room_id: room_id.clone(),
                        user_id,
                        user_name: None,
                        is_typing: false,
                    };
                    
                    for &uid in users {
                        if uid != user_id {
                            act.send_message_to_user(uid, &stop_typing_msg);
                        }
                    }
                }
            }
        });
        
        // Cleanup rate limiter for disconnected users every 60 seconds
        ctx.run_interval(Duration::from_secs(60), |act, _ctx| {
            let active_users: Vec<i64> = act.sessions.keys().copied().collect();
            act.rate_limiter.cleanup_old_users(&active_users);
            info!("Rate limiter cleanup: {} active users", active_users.len());
        });
    }
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        let session_id = Uuid::new_v4().to_string();
        info!("User {} connecting with session {}", msg.user_id, session_id);
        
        let sessions = self.sessions.entry(msg.user_id).or_insert_with(Vec::new);
        
        // Check if user has reached connection limit
        if sessions.len() >= MAX_CONNECTIONS_PER_USER {
            warn!(
                "User {} has reached connection limit ({}). Closing oldest session.",
                msg.user_id, MAX_CONNECTIONS_PER_USER
            );
            
            // Remove and close oldest session (first in list)
            if let Some(oldest_session) = sessions.first() {
                info!("Closing session {} for user {}", oldest_session.session_id, msg.user_id);
                
                // Send replacement message to oldest session
                let _ = oldest_session.addr.do_send(WsResponseMessage(
                    WsResponse::ConnectionReplaced {
                        message: "Phiên của bạn đã bị thay thế bởi một đăng nhập mới. Vui lòng tải lại trang.".to_string(),
                    }
                ));
            }
            
            // Remove oldest session
            sessions.remove(0);
        }
        
        // Add new session
        sessions.push(SessionInfo {
            addr: msg.addr,
            connected_at: Instant::now(),
            session_id: session_id.clone(),
        });
        
        info!(
            "User {} now has {} active session(s). New session: {}",
            msg.user_id,
            sessions.len(),
            session_id
        );
        
        // Initialize user_rooms entry
        self.user_rooms.entry(msg.user_id).or_insert_with(HashSet::new);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Context<Self>) {
        info!("User {} session {} disconnecting", msg.user_id, msg.session_id);
        
        // Remove specific session
        if let Some(sessions) = self.sessions.get_mut(&msg.user_id) {
            sessions.retain(|s| s.session_id != msg.session_id);
            
            info!(
                "User {} now has {} active session(s) after disconnect",
                msg.user_id,
                sessions.len()
            );
            
            // If user has no more sessions, clean up and broadcast offline
            if sessions.is_empty() {
                info!("User {} has no more active sessions, broadcasting offline", msg.user_id);
                self.sessions.remove(&msg.user_id);
                
                // Update last_seen in Redis
                let user_id = msg.user_id;
                let mut redis_conn = self.redis_conn.clone();
                
                ctx.spawn(
                    async move {
                        // Store last_seen in Redis with 30 day TTL
                        let key = format!("user:{}:last_seen", user_id);
                        let now = chrono::Utc::now().to_rfc3339();
                        let _: Result<(), redis::RedisError> = redis_conn
                            .set_ex(&key, now, 30 * 24 * 60 * 60)
                            .await;
                    }
                    .into_actor(self),
                );
                
                // Get all rooms user was in
                let user_room_ids: Vec<String> = self.user_rooms
                    .get(&msg.user_id)
                    .map(|rooms| rooms.iter().cloned().collect())
                    .unwrap_or_default();
                
                // Broadcast offline status to all rooms
                for room_id in &user_room_ids {
                    if let Some(users) = self.rooms.get(room_id) {
                        let online_users: Vec<i64> = users.iter()
                            .filter(|&&uid| uid != msg.user_id && self.sessions.contains_key(&uid))
                            .copied()
                            .collect();
                        
                        // Notify remaining users
                        for &user_id in &online_users {
                            self.send_message_to_user(user_id, &WsResponse::UserOffline {
                                user_id: msg.user_id,
                            });
                        }
                    }
                }
                
                // Remove from all rooms
                for room_id in user_room_ids {
                    if let Some(users) = self.rooms.get_mut(&room_id) {
                        users.remove(&msg.user_id);
                        if users.is_empty() {
                            self.rooms.remove(&room_id);
                        }
                    }
                }
                
                // Clean up typing indicators
                for typing_map in self.typing_users.values_mut() {
                    typing_map.remove(&msg.user_id);
                }
                
                self.user_rooms.remove(&msg.user_id);
            }
        }
    }
}

impl Handler<JoinRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) {
        info!("User {} joining room {}", msg.user_id, msg.room_id);
        
        // Add user to room
        self.rooms
            .entry(msg.room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.user_id);
        
        // Track user's rooms
        self.user_rooms
            .entry(msg.user_id)
            .or_insert_with(HashSet::new)
            .insert(msg.room_id.clone());
        
        // Get all online users in this room
        let online_users: Vec<i64> = self.rooms
            .get(&msg.room_id)
            .map(|users| {
                users.iter()
                    .filter(|&&uid| self.sessions.contains_key(&uid))
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        
        // Send room presence to joining user
        self.send_message_to_user(msg.user_id, &WsResponse::RoomPresence {
            room_id: msg.room_id.clone(),
            online_users: online_users.clone(),
        });
        
        // Notify other users in this room that user is online
        for &other_user_id in &online_users {
            if other_user_id != msg.user_id {
                info!("Notifying user {} that user {} is online", other_user_id, msg.user_id);
                self.send_message_to_user(other_user_id, &WsResponse::UserOnline {
                    user_id: msg.user_id,
                    user_name: format!("User {}", msg.user_id),
                });
            }
        }
        
        // Send confirmation to user
        self.send_message_to_user(msg.user_id, &WsResponse::Joined {
            room_id: msg.room_id.clone(),
        });
    }
}

impl Handler<LeaveRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _: &mut Context<Self>) {
        info!("User {} leaving room {}", msg.user_id, msg.room_id);
        
        if let Some(users) = self.rooms.get_mut(&msg.room_id) {
            users.remove(&msg.user_id);
            if users.is_empty() {
                self.rooms.remove(&msg.room_id);
            }
        }

        let response = WsResponse::Left {
            room_id: msg.room_id,
        };
        self.send_message_to_user(msg.user_id, &response);
    }
}

impl Handler<SendMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _: &mut Context<Self>) {
        self.send_message_to_user(msg.user_id, &msg.message);
    }
}

impl Handler<BroadcastToRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToRoom, ctx: &mut Context<Self>) {
        // ONLY publish to Redis, do NOT broadcast locally
        // Let Redis listener handle broadcasting to ALL instances (including this one)
        let redis_channel = format!("chat:room:{}", msg.room_id);
        let mut redis_conn = self.redis_conn.clone();
        let message = msg.message.clone();
        
        ctx.spawn(
            async move {
                if let Ok(json) = serde_json::to_string(&message) {
                    let _: Result<(), redis::RedisError> = redis_conn.publish(&redis_channel, json).await;
                }
            }
            .into_actor(self),
        );
    }
}

// Handler for local-only broadcast (used by Redis listener)
impl Handler<BroadcastToRoomLocal> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToRoomLocal, _: &mut Context<Self>) {
        // Only broadcast to local connections, do NOT publish to Redis
        if let Some(users) = self.rooms.get(&msg.room_id) {
            for user_id in users {
                if let Some(exclude) = msg.exclude_user {
                    if *user_id == exclude {
                        continue;
                    }
                }
                self.send_message_to_user(*user_id, &msg.message);
            }
        }
    }
}

impl Handler<GetConnectionCount> for ChatServer {
    type Result = usize;

    fn handle(&mut self, _msg: GetConnectionCount, _: &mut Context<Self>) -> Self::Result {
        // Count total number of sessions across all users
        self.sessions.values().map(|sessions| sessions.len()).sum()
    }
}

impl Handler<BroadcastToUsers> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: BroadcastToUsers, _: &mut Context<Self>) {
        info!("Broadcasting to {} users: {:?}", msg.user_ids.len(), msg.user_ids);
        
        // Broadcast message to specific users
        for user_id in msg.user_ids {
            if self.sessions.contains_key(&user_id) {
                info!("Sending message to user {}", user_id);
                self.send_message_to_user(user_id, &msg.message);
            } else {
                info!("User {} not connected to WebSocket", user_id);
            }
        }
    }
}

impl Handler<UserTyping> for ChatServer {
    type Result = ();
    
    fn handle(&mut self, msg: UserTyping, _: &mut Context<Self>) {
        // Track typing timestamp
        self.typing_users
            .entry(msg.room_id)
            .or_insert_with(HashMap::new)
            .insert(msg.user_id, Instant::now());
    }
}

impl Handler<CheckRateLimit> for ChatServer {
    type Result = Result<(), f64>;
    
    fn handle(&mut self, msg: CheckRateLimit, _: &mut Context<Self>) -> Self::Result {
        use super::rate_limiter::EventType;
        
        let event_type = match msg.event_type.as_str() {
            "message" => EventType::Message,
            "typing" => EventType::Typing,
            "room_action" => EventType::RoomAction,
            _ => return Ok(()), // Unknown event type, allow it
        };
        
        self.rate_limiter.check_rate_limit(msg.user_id, event_type)
    }
}
