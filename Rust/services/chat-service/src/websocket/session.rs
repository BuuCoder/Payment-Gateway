use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use sqlx;

use super::messages::*;
use super::server::ChatServer;
use crate::domain::Message;
use crate::repo::{MessageRepository, RoomRepository};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsSession {
    pub id: i64,
    pub user_id: i64,
    pub session_id: String,
    pub hb: Instant,
    pub server_addr: Addr<ChatServer>,
    pub message_repo: MessageRepository,
    pub room_repo: RoomRepository,
}

impl WsSession {
    pub fn new(
        user_id: i64,
        server_addr: Addr<ChatServer>,
        message_repo: MessageRepository,
        room_repo: RoomRepository,
    ) -> Self {
        Self {
            id: rand::random::<i64>(),
            user_id,
            session_id: uuid::Uuid::new_v4().to_string(),
            hb: Instant::now(),
            server_addr,
            message_repo,
            room_repo,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("WebSocket Client heartbeat failed, disconnecting!");
                act.server_addr.do_send(Disconnect {
                    user_id: act.user_id,
                    session_id: act.session_id.clone(),
                });
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    fn handle_message(&mut self, msg: WsMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WsMessage::Message {
                room_id,
                content,
                message_type,
                metadata,
            } => {
                let user_id = self.user_id;
                let server_addr = self.server_addr.clone();
                
                // Check rate limit first
                let rate_check = server_addr.send(CheckRateLimit {
                    user_id,
                    event_type: "message".to_string(),
                });
                
                let message_repo = self.message_repo.clone();
                let room_repo = self.room_repo.clone();

                let fut = async move {
                    // Wait for rate limit check
                    match rate_check.await {
                        Ok(Ok(())) => {
                            // Rate limit OK, proceed
                        }
                        Ok(Err(retry_after)) => {
                            // Rate limit exceeded
                            return Err(format!("rate_limit:{}:{}", retry_after, "Bạn đang gửi tin nhắn quá nhanh. Vui lòng đợi {} giây."));
                        }
                        Err(e) => {
                            error!("Failed to check rate limit: {}", e);
                            return Err("Internal error".to_string());
                        }
                    }
                    
                    // Check if user is member of room
                    match room_repo.is_member(&room_id, user_id).await {
                        Ok(true) => {
                            // Create message
                            let mut message = Message::new(
                                room_id.clone(),
                                user_id,
                                content.clone(),
                                message_type.clone(),
                            );

                            if let Some(meta) = metadata {
                                message.metadata = Some(serde_json::to_string(&meta).unwrap_or_default());
                            }

                            // Save to database
                            if let Err(e) = message_repo.create_message(&message).await {
                                error!("Failed to save message: {}", e);
                                return Err(format!("Failed to save message: {}", e));
                            }

                            // Update room's last_message_at
                            if let Err(e) = room_repo.update_last_message_at(&room_id).await {
                                error!("Failed to update last_message_at: {}", e);
                            }

                            // Unhide room for all members who have hidden it
                            if let Err(e) = room_repo.unhide_room_for_members(&room_id).await {
                                error!("Failed to unhide room: {}", e);
                            }

                            // Get sender name from database
                            let sender_name = sqlx::query_scalar::<_, String>(
                                "SELECT name FROM users WHERE id = ?"
                            )
                            .bind(user_id)
                            .fetch_optional(&message_repo.pool)
                            .await
                            .ok()
                            .flatten();

                            // Broadcast to room
                            let response = WsResponse::Message {
                                id: message.id.clone(),
                                room_id: room_id.clone(),
                                sender_id: user_id,
                                sender_name,
                                content: message.content.clone(),
                                message_type: message.message_type.clone(),
                                metadata: message.metadata.as_ref().and_then(|m| serde_json::from_str(m).ok()),
                                created_at: message.created_at.to_rfc3339(),
                            };

                            server_addr.do_send(BroadcastToRoom {
                                room_id: room_id.clone(),
                                message: response,
                                exclude_user: None,
                            });

                            // Send room_updated notification
                            let room_updated = WsResponse::RoomUpdated {
                                room_id: room_id.clone(),
                                last_message_at: chrono::Utc::now().to_rfc3339(),
                            };

                            server_addr.do_send(BroadcastToRoom {
                                room_id: room_id.clone(),
                                message: room_updated,
                                exclude_user: None,
                            });

                            // Calculate and send unread updates to all members except sender
                            if let Ok(members) = room_repo.get_room_members(&room_id).await {
                                for member in members {
                                    if member.user_id != user_id && member.left_at.is_none() {
                                        if let Ok(unread_count) = room_repo.get_unread_count(&room_id, member.user_id).await {
                                            let unread_notification = WsResponse::UnreadUpdated {
                                                room_id: room_id.clone(),
                                                unread_count,
                                            };

                                            server_addr.do_send(BroadcastToUsers {
                                                user_ids: vec![member.user_id],
                                                message: unread_notification,
                                            });
                                        }
                                    }
                                }
                            }

                            Ok(())
                        }
                        Ok(false) => Err("Not a member of this room".to_string()),
                        Err(e) => Err(format!("Database error: {}", e)),
                    }
                };

                ctx.spawn(fut.into_actor(self).map(|result, _act, ctx| {
                    if let Err(err) = result {
                        // Check if it's a rate limit error
                        if err.starts_with("rate_limit:") {
                            let parts: Vec<&str> = err.splitn(3, ':').collect();
                            if parts.len() == 3 {
                                let retry_after: f64 = parts[1].parse().unwrap_or(5.0);
                                let message = parts[2].replace("{}", &format!("{:.0}", retry_after.ceil()));
                                let response = WsResponse::RateLimitExceeded {
                                    event_type: "message".to_string(),
                                    retry_after,
                                    message,
                                };
                                ctx.text(serde_json::to_string(&response).unwrap_or_default());
                            }
                        } else {
                            let response = WsResponse::Error { message: err };
                            ctx.text(serde_json::to_string(&response).unwrap_or_default());
                        }
                    }
                }));
            }
            WsMessage::JoinRoom { room_id } => {
                let user_id = self.user_id;
                let server_addr = self.server_addr.clone();
                let room_repo = self.room_repo.clone();
                
                // Check rate limit
                let rate_check = server_addr.send(CheckRateLimit {
                    user_id,
                    event_type: "room_action".to_string(),
                });

                let fut = async move {
                    // Wait for rate limit check
                    match rate_check.await {
                        Ok(Ok(())) => {}
                        Ok(Err(retry_after)) => {
                            return Err(format!("rate_limit:{}:Bạn đang thực hiện hành động quá nhanh. Vui lòng đợi {{}} giây.", retry_after));
                        }
                        Err(e) => {
                            error!("Failed to check rate limit: {}", e);
                            return Err("Internal error".to_string());
                        }
                    }
                    
                    match room_repo.is_member(&room_id, user_id).await {
                        Ok(true) => {
                            server_addr.do_send(JoinRoom { user_id, room_id });
                            Ok(())
                        }
                        Ok(false) => Err("Not a member of this room".to_string()),
                        Err(e) => Err(format!("Database error: {}", e)),
                    }
                };

                ctx.spawn(fut.into_actor(self).map(|result, _, ctx| {
                    if let Err(err) = result {
                        if err.starts_with("rate_limit:") {
                            let parts: Vec<&str> = err.splitn(3, ':').collect();
                            if parts.len() == 3 {
                                let retry_after: f64 = parts[1].parse().unwrap_or(5.0);
                                let message = parts[2].replace("{}", &format!("{:.0}", retry_after.ceil()));
                                let response = WsResponse::RateLimitExceeded {
                                    event_type: "room_action".to_string(),
                                    retry_after,
                                    message,
                                };
                                ctx.text(serde_json::to_string(&response).unwrap_or_default());
                            }
                        } else {
                            let response = WsResponse::Error { message: err };
                            ctx.text(serde_json::to_string(&response).unwrap_or_default());
                        }
                    }
                }));
            }
            WsMessage::LeaveRoom { room_id } => {
                self.server_addr.do_send(LeaveRoom {
                    user_id: self.user_id,
                    room_id,
                });
            }
            WsMessage::Typing { room_id, is_typing } => {
                // Check rate limit for typing events
                let user_id = self.user_id;
                let server_addr = self.server_addr.clone();
                
                let rate_check = server_addr.send(CheckRateLimit {
                    user_id,
                    event_type: "typing".to_string(),
                });
                
                let fut = async move {
                    match rate_check.await {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(retry_after)) => Err((retry_after, "typing")),
                        Err(_) => Ok(()), // Allow on error
                    }
                };
                
                ctx.spawn(fut.into_actor(self).map(move |result, act, ctx| {
                    match result {
                        Ok(()) => {
                            // Rate limit OK, proceed
                            if is_typing {
                                // Track typing in server
                                act.server_addr.do_send(UserTyping {
                                    user_id: act.user_id,
                                    room_id: room_id.clone(),
                                });
                            }
                            
                            let response = WsResponse::Typing {
                                room_id: room_id.clone(),
                                user_id: act.user_id,
                                user_name: None,
                                is_typing,
                            };

                            act.server_addr.do_send(BroadcastToRoom {
                                room_id,
                                message: response,
                                exclude_user: Some(act.user_id),
                            });
                        }
                        Err((retry_after, event_type)) => {
                            // Rate limit exceeded - silently ignore for typing
                            // (typing is not critical, no need to show error to user)
                            warn!("Rate limit exceeded for user {} on {}", act.user_id, event_type);
                        }
                    }
                }));
            }
            WsMessage::Ping => {
                let response = WsResponse::Pong;
                ctx.text(serde_json::to_string(&response).unwrap_or_default());
            }
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.server_addr
            .send(Connect {
                addr: addr.recipient(),
                user_id: self.user_id,
            })
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .wait(ctx);

        info!("WebSocket session started for user {}", self.user_id);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server_addr.do_send(Disconnect {
            user_id: self.user_id,
            session_id: self.session_id.clone(),
        });
        info!("WebSocket session {} stopped for user {}", self.session_id, self.user_id);
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.hb = Instant::now();
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(msg) => self.handle_message(msg, ctx),
                    Err(e) => {
                        error!("Failed to parse message: {}", e);
                        let response = WsResponse::Error {
                            message: "Invalid message format".to_string(),
                        };
                        ctx.text(serde_json::to_string(&response).unwrap_or_default());
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                warn!("Binary messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<WsResponseMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsResponseMessage, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg.0) {
            ctx.text(json);
        }
    }
}
