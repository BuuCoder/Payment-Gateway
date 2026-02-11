use actix_web::{web, HttpResponse, Responder};
use crate::service::user_service::UserService;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "core-service"
    }))
}

pub async fn get_users(service: web::Data<UserService>) -> impl Responder {
    match service.get_all_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}

pub async fn get_user(
    service: web::Data<UserService>,
    user_id: web::Path<i32>,
) -> impl Responder {
    match service.get_user_by_id(user_id.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) if e == "User not found" => HttpResponse::NotFound().json(serde_json::json!({
            "error": e
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e
        })),
    }
}
