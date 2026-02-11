use actix_web::{HttpResponse, Responder};

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "auth-service"
    }))
}

pub async fn login() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Login endpoint - TODO"
    }))
}

pub async fn register() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Register endpoint - TODO"
    }))
}
