use actix::Addr;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::domain::{CreateDirectRoomRequest, CreateRoomRequest, Room, RoomMemberResponse, RoomResponse};
use crate::repo::{MessageRepository, RoomRepository};
use crate::websocket::{ChatServer, WsSession};
use authz::jwt::Claims;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    service: String,
    instance: String,
    version: String,
    database: String,
    redis: String,
    websocket_connections: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub chat_server: Addr<ChatServer>,
    pub message_repo: MessageRepository,
    pub room_repo: RoomRepository,
}

// Health check endpoint (no auth required)
pub async fn health_check(state: web::Data<AppState>) -> Result<HttpResponse> {
    // Check database connection
    let db_status = match state.room_repo.pool.acquire().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Get WebSocket connection count
    let ws_count = state.chat_server.send(crate::websocket::GetConnectionCount).await
        .unwrap_or(0);

    // Get instance name from environment
    let instance = std::env::var("SERVICE_NAME")
        .unwrap_or_else(|_| "unknown".to_string());

    let response = HealthResponse {
        status: if db_status == "healthy" {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        service: "chat-service".to_string(),
        instance,
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status.to_string(),
        redis: "not-checked".to_string(), // Redis check via WebSocket server
        websocket_connections: ws_count,
    };

    Ok(HttpResponse::Ok().json(response))
}

// Helper function to extract claims from request
fn get_claims(req: &HttpRequest) -> Result<Claims> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))
}

// WebSocket connection handler
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    info!("WebSocket connection request from user {}", claims.user_id);

    let session = WsSession::new(
        claims.user_id as i64,
        state.chat_server.clone(),
        state.message_repo.clone(),
        state.room_repo.clone(),
    );

    ws::start(session, &req, stream)
}

// Create a new room (direct or group)
pub async fn create_room(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateRoomRequest>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Validate room type
    if body.room_type != "direct" && body.room_type != "group" {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid room type. Must be 'direct' or 'group'"
        })));
    }

    // For direct rooms, check if room already exists
    if body.room_type == "direct" {
        if body.member_ids.len() != 1 {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Direct rooms must have exactly one other member"
            })));
        }

        let other_user_id = body.member_ids[0];
        match state.room_repo.find_direct_room(user_id, other_user_id).await {
            Ok(Some(room)) => {
                // Room already exists, return it
                let members = state.room_repo.get_room_members(&room.id).await
                    .map_err(|e| {
                        error!("Failed to get room members: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to get room members")
                    })?;

                let response = RoomResponse {
                    id: room.id,
                    name: room.name,
                    room_type: room.room_type,
                    created_by: room.created_by,
                    members: members.into_iter().map(|m| RoomMemberResponse {
                        user_id: m.user_id,
                        role: m.role,
                        joined_at: m.joined_at,
                    }).collect(),
                    created_at: room.created_at,
                };

                return Ok(HttpResponse::Ok().json(response));
            }
            Ok(None) => {
                // Continue to create new room
            }
            Err(e) => {
                error!("Failed to check existing room: {}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to check existing room"
                })));
            }
        }
    }

    // Create new room
    let room = Room::new(body.name.clone(), body.room_type.clone(), user_id);

    state.room_repo.create_room(&room).await.map_err(|e| {
        error!("Failed to create room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create room")
    })?;

    // Add creator as admin
    state.room_repo.add_member(&room.id, user_id, "admin").await.map_err(|e| {
        error!("Failed to add creator to room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to add creator to room")
    })?;

    // Add other members
    for member_id in &body.member_ids {
        if *member_id != user_id {
            state.room_repo.add_member(&room.id, *member_id, "member").await.map_err(|e| {
                error!("Failed to add member to room: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to add member to room")
            })?;
        }
    }

    // Get all members
    let members = state.room_repo.get_room_members(&room.id).await.map_err(|e| {
        error!("Failed to get room members: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get room members")
    })?;

    let response = RoomResponse {
        id: room.id,
        name: room.name,
        room_type: room.room_type,
        created_by: room.created_by,
        members: members.into_iter().map(|m| RoomMemberResponse {
            user_id: m.user_id,
            role: m.role,
            joined_at: m.joined_at,
        }).collect(),
        created_at: room.created_at,
    };

    Ok(HttpResponse::Created().json(response))
}

// Create or get direct chat room (1-1 chat)
pub async fn create_direct_room(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateDirectRoomRequest>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;
    let other_user_id = body.other_user_id;

    // Check if room already exists
    match state.room_repo.find_direct_room(user_id, other_user_id).await {
        Ok(Some(room)) => {
            // Room already exists, return it
            info!("Returning existing direct room {} between users {} and {}", room.id, user_id, other_user_id);
            
            let members = state.room_repo.get_room_members(&room.id).await
                .map_err(|e| {
                    error!("Failed to get room members: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to get room members")
                })?;

            let response = RoomResponse {
                id: room.id,
                name: room.name,
                room_type: room.room_type,
                created_by: room.created_by,
                members: members.into_iter().map(|m| RoomMemberResponse {
                    user_id: m.user_id,
                    role: m.role,
                    joined_at: m.joined_at,
                }).collect(),
                created_at: room.created_at,
            };

            return Ok(HttpResponse::Ok().json(response));
        }
        Ok(None) => {
            // Continue to create new room
            info!("Creating new direct room between users {} and {}", user_id, other_user_id);
        }
        Err(e) => {
            error!("Failed to check existing room: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to check existing room"
            })));
        }
    }

    // Create new direct room
    let room = Room::new(None, "direct".to_string(), user_id);

    state.room_repo.create_room(&room).await.map_err(|e| {
        error!("Failed to create room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create room")
    })?;

    // Add creator as admin
    state.room_repo.add_member(&room.id, user_id, "admin").await.map_err(|e| {
        error!("Failed to add creator to room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to add creator to room")
    })?;

    // Add other user as member
    state.room_repo.add_member(&room.id, other_user_id, "member").await.map_err(|e| {
        error!("Failed to add member to room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to add member to room")
    })?;

    let members = state.room_repo.get_room_members(&room.id).await
        .map_err(|e| {
            error!("Failed to get room members: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get room members")
        })?;

    let response = RoomResponse {
        id: room.id.clone(),
        name: room.name,
        room_type: room.room_type,
        created_by: room.created_by,
        members: members.into_iter().map(|m| RoomMemberResponse {
            user_id: m.user_id,
            role: m.role,
            joined_at: m.joined_at,
        }).collect(),
        created_at: room.created_at,
    };

    info!("Created direct room {} between users {} and {}", room.id, user_id, other_user_id);
    Ok(HttpResponse::Created().json(response))
}

// Get user's rooms
pub async fn get_user_rooms(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    let rooms = state.room_repo.get_user_rooms(user_id).await.map_err(|e| {
        error!("Failed to get user rooms: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get user rooms")
    })?;

    Ok(HttpResponse::Ok().json(rooms))
}

// Get room details
pub async fn get_room(
    req: HttpRequest,
    state: web::Data<AppState>,
    room_id: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Check if user is member
    let is_member = state.room_repo.is_member(&room_id, user_id).await.map_err(|e| {
        error!("Failed to check room membership: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to check room membership")
    })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    let room = state.room_repo.get_room(&room_id).await.map_err(|e| {
        error!("Failed to get room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get room")
    })?;

    match room {
        Some(room) => {
            let members = state.room_repo.get_room_members(&room.id).await.map_err(|e| {
                error!("Failed to get room members: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to get room members")
            })?;

            let response = RoomResponse {
                id: room.id,
                name: room.name,
                room_type: room.room_type,
                created_by: room.created_by,
                members: members.into_iter().map(|m| RoomMemberResponse {
                    user_id: m.user_id,
                    role: m.role,
                    joined_at: m.joined_at,
                }).collect(),
                created_at: room.created_at,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Room not found"
        }))),
    }
}

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    limit: Option<i64>,
    before_id: Option<String>,
}

// Get room messages
pub async fn get_room_messages(
    req: HttpRequest,
    state: web::Data<AppState>,
    room_id: web::Path<String>,
    query: web::Query<GetMessagesQuery>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Check if user is member
    let is_member = state.room_repo.is_member(&room_id, user_id).await.map_err(|e| {
        error!("Failed to check room membership: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to check room membership")
    })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let messages = state
        .message_repo
        .get_room_messages(&room_id, limit, query.before_id.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to get messages: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get messages")
        })?;

    let responses: Vec<_> = messages.into_iter().map(|m| m.to_response()).collect();

    Ok(HttpResponse::Ok().json(responses))
}
