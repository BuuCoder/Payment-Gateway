use actix_web::{web, HttpResponse, Responder};
use authz::Claims;
use crate::service::user_service::UserService;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "core-service"
    }))
}

pub async fn get_users(
    _claims: web::ReqData<Claims>,
    service: web::Data<UserService>,
) -> impl Responder {
    match service.get_all_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            tracing::error!("Failed to get users: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch users"
            }))
        }
    }
}

pub async fn get_user(
    _claims: web::ReqData<Claims>,
    service: web::Data<UserService>,
    user_id: web::Path<i32>,
) -> impl Responder {
    match service.get_user_by_id(user_id.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))
        }
    }
}
