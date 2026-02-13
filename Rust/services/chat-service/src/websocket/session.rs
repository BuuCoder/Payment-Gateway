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
                let message_repo = self.message_repo.clone();
                let room_repo = self.room_repo.clone();

                let fut = async move {
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
                                room_id,
                                message: response,
                                exclude_user: None,
                            });

                            Ok(())
                        }
                        Ok(false) => Err("Not a member of this room".to_string()),
                        Err(e) => Err(format!("Database error: {}", e)),
                    }
                };

                ctx.spawn(fut.into_actor(self).map(|result, _act, ctx| {
                    if let Err(err) = result {
                        let response = WsResponse::Error { message: err };
                        ctx.text(serde_json::to_string(&response).unwrap_or_default());
                    }
                }));
            }
            WsMessage::JoinRoom { room_id } => {
                let user_id = self.user_id;
                let server_addr = self.server_addr.clone();
                let room_repo = self.room_repo.clone();

                let fut = async move {
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
                        let response = WsResponse::Error { message: err };
                        ctx.text(serde_json::to_string(&response).unwrap_or_default());
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
                let response = WsResponse::Typing {
                    room_id: room_id.clone(),
                    user_id: self.user_id,
                    user_name: None,
                    is_typing,
                };

                self.server_addr.do_send(BroadcastToRoom {
                    room_id,
                    message: response,
                    exclude_user: Some(self.user_id),
                });
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
        });
        info!("WebSocket session stopped for user {}", self.user_id);
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
