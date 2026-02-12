use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::service::AuthService;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: crate::domain::UserPublic,
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        service: "auth-service".to_string(),
    })
}

pub async fn register(
    auth_service: web::Data<AuthService>,
    request: web::Json<RegisterRequest>,
) -> impl Responder {
    match auth_service.register(&request.name, &request.email, &request.password).await {
        Ok((token, user)) => {
            HttpResponse::Created().json(AuthResponse { token, user })
        }
        Err(e) => {
            tracing::error!("Register error: {}", e);
            let error_msg = e.to_string();
            
            if error_msg.contains("already registered") {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": error_msg
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to register user"
                }))
            }
        }
    }
}

pub async fn login(
    auth_service: web::Data<AuthService>,
    request: web::Json<LoginRequest>,
) -> impl Responder {
    match auth_service.login(&request.email, &request.password).await {
        Ok((token, user)) => {
            HttpResponse::Ok().json(AuthResponse { token, user })
        }
        Err(e) => {
            tracing::error!("Login error: {}", e);
            let error_msg = e.to_string();
            
            if error_msg.contains("Invalid credentials") {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials"
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Login failed"
                }))
            }
        }
    }
}
