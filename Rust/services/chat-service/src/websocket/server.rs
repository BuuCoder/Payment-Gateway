use actix::prelude::*;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};
use tracing::{error, info};

use super::messages::*;

pub struct ChatServer {
    // user_id -> session address
    sessions: HashMap<i64, Recipient<WsResponseMessage>>,
    // room_id -> set of user_ids
    rooms: HashMap<String, HashSet<i64>>,
    // Redis connection for pub/sub
    redis_conn: MultiplexedConnection,
}

impl ChatServer {
    pub fn new(redis_conn: MultiplexedConnection) -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            redis_conn,
        }
    }

    fn send_message_to_user(&self, user_id: i64, message: &WsResponse) {
        if let Some(addr) = self.sessions.get(&user_id) {
            let _ = addr.do_send(WsResponseMessage(message.clone()));
        }
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

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ChatServer started");
    }
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!("User {} connected", msg.user_id);
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("User {} disconnected", msg.user_id);
        
        // Remove from all rooms
        let rooms_to_clean: Vec<String> = self
            .rooms
            .iter()
            .filter(|(_, users)| users.contains(&msg.user_id))
            .map(|(room_id, _)| room_id.clone())
            .collect();

        for room_id in rooms_to_clean {
            if let Some(users) = self.rooms.get_mut(&room_id) {
                users.remove(&msg.user_id);
                if users.is_empty() {
                    self.rooms.remove(&room_id);
                }
            }
        }

        self.sessions.remove(&msg.user_id);
    }
}

impl Handler<JoinRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) {
        info!("User {} joining room {}", msg.user_id, msg.room_id);
        
        self.rooms
            .entry(msg.room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.user_id);

        // Send confirmation to user (local only, no Redis publish)
        // Join/leave are local state changes, not cross-instance events
        let response = WsResponse::Joined {
            room_id: msg.room_id.clone(),
        };
        self.send_message_to_user(msg.user_id, &response);
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
        self.sessions.len()
    }
}
