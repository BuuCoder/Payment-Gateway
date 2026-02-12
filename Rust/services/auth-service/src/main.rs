mod api;
mod domain;
mod repo;
mod service;

use actix_web::{web, App, HttpServer};
use common::config::AppConfig;
use repo::UserRepository;
use service::AuthService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = AppConfig::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Initialize layers
    let user_repo = UserRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repo);
    
    let server_address = config.server_address();
    tracing::info!("🔐 Auth Service starting on http://{}", server_address);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .configure(api::routes::configure)
    })
    .bind(&server_address)?
    .run()
    .await
}
