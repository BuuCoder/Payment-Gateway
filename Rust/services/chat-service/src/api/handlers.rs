use actix::Addr;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::domain::{CreateDirectRoomRequest, CreateRoomRequest, Room, RoomMemberResponse, RoomResponse, Message};
use crate::repo::{MessageRepository, RoomRepository, InvitationRepository};
use crate::websocket::{ChatServer, WsSession, BroadcastToUsers, BroadcastToRoom, WsResponse};
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
    pub invitation_repo: InvitationRepository,
    pub redis_cache: common::cache::RedisCache,
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

// Helper function to extract claims from query string (for WebSocket)
fn get_claims_from_query(req: &HttpRequest) -> Result<Claims> {
    use authz::jwt::JwtValidator;
    
    // Try to get from middleware first (header Authorization)
    if let Some(claims) = req.extensions().get::<Claims>() {
        return Ok(claims.clone());
    }
    
    // Fallback: Read token from query string
    let query = req.query_string();
    let token = query
        .split('&')
        .find(|s| s.starts_with("token="))
        .and_then(|s| s.strip_prefix("token="))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No token provided"))?;
    
    // Decode JWT
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "RushTech@2025xAjxh".to_string());
    
    let validator = JwtValidator::new(jwt_secret);
    validator.verify_token(token)
        .map_err(|e| {
            error!("Failed to decode token from query string: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })
}

// WebSocket connection handler
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // Try to get claims from query string (for browser WebSocket)
    let claims = get_claims_from_query(&req)?;
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
                let members_with_users = state.room_repo.get_room_members_with_users(&room.id).await
                    .map_err(|e| {
                        error!("Failed to get room members: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to get room members")
                    })?;

                let response = RoomResponse {
                    id: room.id.clone(),
                    name: room.name.clone(),
                    room_type: room.room_type.clone(),
                    created_by: room.created_by,
                    members: members_with_users.into_iter().map(|(m, user_name, user_email)| RoomMemberResponse {
                        user_id: m.user_id,
                        role: m.role,
                        joined_at: m.joined_at,
                        user_name,
                        user_email,
                    }).collect(),
                    created_at: room.created_at,
                    last_message_at: room.last_message_at,
                    unread_count: None,
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

    // For group chats, create invitations for other members
    // For direct chats, add the other user directly
    if body.room_type == "group" {
        // Create invitations for group members
        for member_id in &body.member_ids {
            if *member_id != user_id {
                match state.invitation_repo.create_invitation(&room.id, *member_id, user_id).await {
                    Ok(invitation_id) => {
                        info!("Created invitation {} for user {} to room {}", invitation_id, member_id, room.id);
                        
                        // Get inviter name
                        let inviter_name = state.room_repo.get_user_name(user_id).await
                            .unwrap_or(None)
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        // Send invitation notification via WebSocket (local instance)
                        let notification = WsResponse::InvitationReceived {
                            invitation_id,
                            room_id: room.id.clone(),
                            room_name: room.name.clone(),
                            invited_by: user_id,
                            invited_by_name: inviter_name.clone(),
                        };
                        
                        state.chat_server.do_send(BroadcastToUsers {
                            user_ids: vec![*member_id],
                            message: notification.clone(),
                        });
                        
                        // Also publish to Redis for cross-instance delivery
                        if let Ok(mut redis_conn) = state.redis_cache.get_connection() {
                            let channel = format!("chat:user:{}", member_id);
                            if let Ok(payload) = serde_json::to_string(&notification) {
                                let _: Result<(), _> = redis::cmd("PUBLISH")
                                    .arg(&channel)
                                    .arg(&payload)
                                    .query(&mut redis_conn);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create invitation for user {}: {}", member_id, e);
                    }
                }
            }
        }
    } else {
        // For direct chats, add other member directly
        for member_id in &body.member_ids {
            if *member_id != user_id {
                state.room_repo.add_member(&room.id, *member_id, "member").await.map_err(|e| {
                    error!("Failed to add member to room: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to add member to room")
                })?;
            }
        }
    }

    // Get all members
    let members_with_users = state.room_repo.get_room_members_with_users(&room.id).await.map_err(|e| {
        error!("Failed to get room members: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get room members")
    })?;

    let response = RoomResponse {
        id: room.id.clone(),
        name: room.name.clone(),
        room_type: room.room_type.clone(),
        created_by: room.created_by,
        members: members_with_users.iter().map(|(m, user_name, user_email)| RoomMemberResponse {
            user_id: m.user_id,
            role: m.role.clone(),
            joined_at: m.joined_at,
            user_name: user_name.clone(),
            user_email: user_email.clone(),
        }).collect(),
        created_at: room.created_at,
        last_message_at: room.last_message_at,
        unread_count: None,
    };

    // Notify all members about new room via WebSocket
    let member_ids: Vec<i64> = members_with_users.iter()
        .map(|(m, _, _)| m.user_id)
        .filter(|id| *id != user_id) // Exclude creator
        .collect();
    
    info!("Notifying {} members about new room {}: {:?}", member_ids.len(), room.id, member_ids);
    
    if !member_ids.is_empty() {
        let notification = WsResponse::RoomCreated {
            room_id: room.id.clone(),
            room_name: room.name.clone(),
            room_type: room.room_type.clone(),
        };
        
        state.chat_server.do_send(BroadcastToUsers {
            user_ids: member_ids.clone(),
            message: notification,
        });
        
        info!("Sent room_created notification to users: {:?}", member_ids);
    }

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
            
            let members_with_users = state.room_repo.get_room_members_with_users(&room.id).await
                .map_err(|e| {
                    error!("Failed to get room members: {}", e);
                    actix_web::error::ErrorInternalServerError("Failed to get room members")
                })?;

            let response = RoomResponse {
                id: room.id.clone(),
                name: room.name.clone(),
                room_type: room.room_type.clone(),
                created_by: room.created_by,
                members: members_with_users.into_iter().map(|(m, user_name, user_email)| RoomMemberResponse {
                    user_id: m.user_id,
                    role: m.role,
                    joined_at: m.joined_at,
                    user_name,
                    user_email,
                }).collect(),
                created_at: room.created_at,
                last_message_at: room.last_message_at,
                unread_count: None,
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

    let members_with_users = state.room_repo.get_room_members_with_users(&room.id).await
        .map_err(|e| {
            error!("Failed to get room members: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get room members")
        })?;

    let response = RoomResponse {
        id: room.id.clone(),
        name: room.name,
        room_type: room.room_type,
        created_by: room.created_by,
        members: members_with_users.into_iter().map(|(m, user_name, user_email)| RoomMemberResponse {
            user_id: m.user_id,
            role: m.role,
            joined_at: m.joined_at,
            user_name,
            user_email,
        }).collect(),
        created_at: room.created_at,
        last_message_at: room.last_message_at,
        unread_count: None,
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

    let rooms_with_members = state.room_repo.get_user_rooms_with_members(user_id).await.map_err(|e| {
        error!("Failed to get user rooms: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get user rooms")
    })?;

    let mut responses: Vec<RoomResponse> = Vec::new();
    
    for (room, members_with_users) in rooms_with_members {
        // Calculate unread count
        let unread_count = state.room_repo.get_unread_count(&room.id, user_id).await
            .unwrap_or(0);
        
        responses.push(RoomResponse {
            id: room.id,
            name: room.name,
            room_type: room.room_type,
            created_by: room.created_by,
            members: members_with_users.into_iter().map(|(m, user_name, user_email)| RoomMemberResponse {
                user_id: m.user_id,
                role: m.role,
                joined_at: m.joined_at,
                user_name,
                user_email,
            }).collect(),
            created_at: room.created_at,
            last_message_at: room.last_message_at,
            unread_count: Some(unread_count),
        });
    }

    Ok(HttpResponse::Ok().json(responses))
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
    let is_member = state.room_repo.is_member(&*room_id, user_id).await.map_err(|e| {
        error!("Failed to check room membership: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to check room membership")
    })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    let room = state.room_repo.get_room(&*room_id).await.map_err(|e| {
        error!("Failed to get room: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to get room")
    })?;

    match room {
        Some(room) => {
            let members_with_users = state.room_repo.get_room_members_with_users(&room.id).await.map_err(|e| {
                error!("Failed to get room members: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to get room members")
            })?;

            let response = RoomResponse {
                id: room.id,
                name: room.name,
                room_type: room.room_type,
                created_by: room.created_by,
                members: members_with_users.into_iter().map(|(m, user_name, user_email)| RoomMemberResponse {
                    user_id: m.user_id,
                    role: m.role,
                    joined_at: m.joined_at,
                    user_name,
                    user_email,
                }).collect(),
                created_at: room.created_at,
                last_message_at: room.last_message_at,
                unread_count: None,
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
    let is_member = state.room_repo.is_member(&*room_id, user_id).await.map_err(|e| {
        error!("Failed to check room membership: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to check room membership")
    })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let messages_with_users = state
        .message_repo
        .get_room_messages_with_users(&*room_id, limit, query.before_id.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to get messages: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get messages")
        })?;

    let responses: Vec<_> = messages_with_users
        .into_iter()
        .map(|(m, sender_name)| {
            let mut response = m.to_response();
            response.sender_name = sender_name;
            response
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

// ============================================
// Invitation Handlers
// ============================================

// Get user's pending invitations
pub async fn get_invitations(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    let invitations = state.invitation_repo
        .get_user_invitations(user_id, Some("pending"))
        .await
        .map_err(|e| {
            error!("Failed to get invitations: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get invitations")
        })?;

    Ok(HttpResponse::Ok().json(invitations))
}

// Accept invitation
pub async fn accept_invitation(
    req: HttpRequest,
    state: web::Data<AppState>,
    invitation_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Get invitation
    let invitation = state.invitation_repo
        .get_invitation(*invitation_id)
        .await
        .map_err(|e| {
            error!("Failed to get invitation: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get invitation")
        })?;

    let invitation = match invitation {
        Some(inv) => inv,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Invitation not found"
            })));
        }
    };

    // Verify user owns this invitation
    if invitation.user_id != user_id {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not your invitation"
        })));
    }

    // Check if already accepted or declined
    if invitation.status != "pending" {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invitation already {}", invitation.status)
        })));
    }

    // Update invitation status
    state.invitation_repo
        .update_invitation_status(*invitation_id, "accepted")
        .await
        .map_err(|e| {
            error!("Failed to update invitation: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update invitation")
        })?;

    // Add user to room
    state.room_repo
        .add_member(&invitation.room_id, user_id, "member")
        .await
        .map_err(|e| {
            error!("Failed to add member to room: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to add member to room")
        })?;

    // Get room details
    let room = state.room_repo
        .get_room(&invitation.room_id)
        .await
        .map_err(|e| {
            error!("Failed to get room: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get room")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Room not found"))?;

    let members_with_users = state.room_repo
        .get_room_members_with_users(&room.id)
        .await
        .map_err(|e| {
            error!("Failed to get room members: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get room members")
        })?;

    let response = RoomResponse {
        id: room.id.clone(),
        name: room.name.clone(),
        room_type: room.room_type.clone(),
        created_by: room.created_by,
        members: members_with_users.iter().map(|(m, user_name, user_email)| RoomMemberResponse {
            user_id: m.user_id,
            role: m.role.clone(),
            joined_at: m.joined_at,
            user_name: user_name.clone(),
            user_email: user_email.clone(),
        }).collect(),
        created_at: room.created_at,
        last_message_at: room.last_message_at,
        unread_count: None,
    };

    // Get user name
    let user_name = state.room_repo
        .get_user_name(user_id)
        .await
        .map_err(|e| {
            error!("Failed to get user name: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get user name")
        })?
        .unwrap_or_else(|| "Unknown".to_string());

    // Create system message for join event
    let system_message_content = format!("{} đã tham gia nhóm", user_name);
    let system_message = Message::new(
        room.id.clone(),
        user_id, // Use joining user's ID but mark as system
        system_message_content.clone(),
        "system".to_string(),
    );
    
    state.message_repo
        .create_message(&system_message)
        .await
        .map_err(|e| {
            error!("Failed to create system message: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create system message")
        })?;

    // Broadcast system message to room
    let system_ws_message = WsResponse::Message {
        id: system_message.id.clone(),
        room_id: room.id.clone(),
        sender_id: user_id,
        sender_name: Some("System".to_string()),
        content: system_message_content,
        message_type: "system".to_string(),
        metadata: None,
        created_at: system_message.created_at.to_rfc3339(),
    };

    state.chat_server.do_send(BroadcastToRoom {
        room_id: room.id.clone(),
        message: system_ws_message,
        exclude_user: None,
    });

    // Notify all room members about new member
    let member_ids: Vec<i64> = members_with_users.iter()
        .map(|(m, _, _)| m.user_id)
        .filter(|id| *id != user_id)
        .collect();

    if !member_ids.is_empty() {
        let notification = WsResponse::MemberJoined {
            room_id: room.id.clone(),
            user_id,
            user_name: user_name.clone(),
        };

        state.chat_server.do_send(BroadcastToUsers {
            user_ids: member_ids,
            message: notification,
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "room": response
    })))
}

// Decline invitation
pub async fn decline_invitation(
    req: HttpRequest,
    state: web::Data<AppState>,
    invitation_id: web::Path<i64>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Get invitation
    let invitation = state.invitation_repo
        .get_invitation(*invitation_id)
        .await
        .map_err(|e| {
            error!("Failed to get invitation: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get invitation")
        })?;

    let invitation = match invitation {
        Some(inv) => inv,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Invitation not found"
            })));
        }
    };

    // Verify user owns this invitation
    if invitation.user_id != user_id {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not your invitation"
        })));
    }

    // Check if already accepted or declined
    if invitation.status != "pending" {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Invitation already {}", invitation.status)
        })));
    }

    // Update invitation status
    state.invitation_repo
        .update_invitation_status(*invitation_id, "declined")
        .await
        .map_err(|e| {
            error!("Failed to update invitation: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to update invitation")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Invitation declined"
    })))
}

// ============================================
// Room Management Handlers
// ============================================

// Leave group
pub async fn leave_room(
    req: HttpRequest,
    state: web::Data<AppState>,
    room_id: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Get room
    let room = state.room_repo
        .get_room(&*room_id)
        .await
        .map_err(|e| {
            error!("Failed to get room: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get room")
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Room not found"))?;

    // Only allow leaving group chats
    if room.room_type != "group" {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot leave direct chat"
        })));
    }

    // Check if user is member
    let is_member = state.room_repo
        .is_member(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to check membership: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to check membership")
        })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    // Get all members
    let members = state.room_repo
        .get_room_members(&*room_id)
        .await
        .map_err(|e| {
            error!("Failed to get members: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get members")
        })?;

    // Check if user is last admin
    let admins: Vec<_> = members.iter()
        .filter(|m| m.role == "admin" && m.left_at.is_none())
        .collect();

    let active_members: Vec<_> = members.iter()
        .filter(|m| m.left_at.is_none())
        .collect();

    if admins.len() == 1 && admins[0].user_id == user_id && active_members.len() > 1 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Cannot leave: you are the last admin. Transfer admin role first."
        })));
    }

    // Leave room
    state.room_repo
        .leave_room(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to leave room: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to leave room")
        })?;

    // Get user name
    let user_name = state.room_repo
        .get_user_name(user_id)
        .await
        .map_err(|e| {
            error!("Failed to get user name: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get user name")
        })?
        .unwrap_or_else(|| "Unknown".to_string());

    // Notify other members
    let member_ids: Vec<i64> = members.iter()
        .filter(|m| m.user_id != user_id && m.left_at.is_none())
        .map(|m| m.user_id)
        .collect();

    if !member_ids.is_empty() {
        let notification = WsResponse::MemberLeft {
            room_id: room_id.to_string(),
            user_id,
            user_name,
        };

        state.chat_server.do_send(BroadcastToUsers {
            user_ids: member_ids,
            message: notification,
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Left group successfully"
    })))
}

// Hide conversation
pub async fn hide_room(
    req: HttpRequest,
    state: web::Data<AppState>,
    room_id: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Check if user is member
    let is_member = state.room_repo
        .is_member(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to check membership: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to check membership")
        })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    // Hide room
    state.room_repo
        .hide_room(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to hide room: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to hide room")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Conversation hidden"
    })))
}

// Mark room as read
pub async fn mark_room_as_read(
    req: HttpRequest,
    state: web::Data<AppState>,
    room_id: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = get_claims(&req)?;
    let user_id = claims.user_id as i64;

    // Check if user is member
    let is_member = state.room_repo
        .is_member(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to check membership: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to check membership")
        })?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Not a member of this room"
        })));
    }

    // Mark as read
    state.room_repo
        .mark_room_as_read(&*room_id, user_id)
        .await
        .map_err(|e| {
            error!("Failed to mark as read: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to mark as read")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Marked as read",
        "last_read_at": chrono::Utc::now().to_rfc3339()
    })))
}
