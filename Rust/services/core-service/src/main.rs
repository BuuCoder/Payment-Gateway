mod api;
mod domain;
mod repo;
mod service;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use common::config::AppConfig;
use repo::user_repo::UserRepository;
use service::user_service::UserService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = AppConfig::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Initialize repository and service
    let user_repository = UserRepository::new(pool);
    let user_service = UserService::new(user_repository);
    
    let server_address = config.server_address();
    tracing::info!("⚙️  Core Service starting on http://{}", server_address);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_service.clone()))
            .configure(api::routes::configure)
    })
    .bind(&server_address)?
    .run()
    .await
}
