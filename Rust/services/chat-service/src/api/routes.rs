use actix_web::web;
use authz::AuthMiddleware;
use common::cache::RedisCache;

use super::handlers::*;
use crate::middleware::RateLimiter;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "RushTech@2025xAjxh".to_string());
    
    // Health check outside /api scope (no auth required)
    cfg.route("/health", web::get().to(health_check));
    
    // Try to create Redis cache for rate limiting
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://redis:6379".to_string());
    
    match RedisCache::new(&redis_url) {
        Ok(redis_cache) => {
            // Rate limiter: 100 requests capacity, 10 requests/second (600/minute)
            let rate_limiter = RateLimiter::new(redis_cache, 100.0, 10.0);
            
            // WebSocket endpoint - tách riêng để tự xử lý auth từ query string
            cfg.route("/api/ws", web::get().to(ws_handler));
            
            cfg.service(
                web::scope("/api")
                    .wrap(rate_limiter)
                    .wrap(AuthMiddleware::new(jwt_secret))
                    // Room endpoints
                    .route("/rooms", web::post().to(create_room))
                    .route("/rooms/direct", web::post().to(create_direct_room))
                    .route("/rooms", web::get().to(get_user_rooms))
                    .route("/rooms/{room_id}", web::get().to(get_room))
                    .route("/rooms/{room_id}/messages", web::get().to(get_room_messages))
                    .route("/rooms/{room_id}/leave", web::post().to(leave_room))
                    .route("/rooms/{room_id}/hide", web::post().to(hide_room))
                    .route("/rooms/{room_id}/mark-read", web::post().to(mark_room_as_read))
                    // Invitation endpoints
                    .route("/invitations", web::get().to(get_invitations))
                    .route("/invitations/{invitation_id}/accept", web::post().to(accept_invitation))
                    .route("/invitations/{invitation_id}/decline", web::post().to(decline_invitation)),
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to create Redis cache for rate limiting: {}. Rate limiting disabled.", e);
            
            // WebSocket endpoint - tách riêng để tự xử lý auth từ query string
            cfg.route("/api/ws", web::get().to(ws_handler));
            
            // Configure routes without rate limiting
            cfg.service(
                web::scope("/api")
                    .wrap(AuthMiddleware::new(jwt_secret))
                    // Room endpoints
                    .route("/rooms", web::post().to(create_room))
                    .route("/rooms/direct", web::post().to(create_direct_room))
                    .route("/rooms", web::get().to(get_user_rooms))
                    .route("/rooms/{room_id}", web::get().to(get_room))
                    .route("/rooms/{room_id}/messages", web::get().to(get_room_messages))
                    .route("/rooms/{room_id}/leave", web::post().to(leave_room))
                    .route("/rooms/{room_id}/hide", web::post().to(hide_room))
                    .route("/rooms/{room_id}/mark-read", web::post().to(mark_room_as_read))
                    // Invitation endpoints
                    .route("/invitations", web::get().to(get_invitations))
                    .route("/invitations/{invitation_id}/accept", web::post().to(accept_invitation))
                    .route("/invitations/{invitation_id}/decline", web::post().to(decline_invitation)),
            );
        }
    }
}
