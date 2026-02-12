mod api;
mod domain;
mod repo;
mod service;
mod middleware;

use actix_web::{web, App, HttpServer};
use common::config::AppConfig;
use common::cache::RedisCache;
use repo::UserRepository;
use service::AuthService;
use middleware::rate_limit::RateLimiter;
use middleware::api_key::ApiKeyAuth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = AppConfig::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Initialize Redis cache for rate limiting
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_cache = RedisCache::new(&redis_url)
        .expect("Failed to connect to Redis");
    
    // Load API keys from environment (comma-separated)
    let api_keys: Vec<String> = std::env::var("AUTH_API_KEYS")
        .unwrap_or_else(|_| {
            tracing::warn!("AUTH_API_KEYS not set, using default key (INSECURE for production!)");
            "dev-key-12345".to_string()
        })
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    tracing::info!("🔑 Loaded {} API key(s)", api_keys.len());
    
    // Initialize layers
    let user_repo = UserRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repo);
    
    // API Key authentication (must have valid key to access)
    let api_key_auth = ApiKeyAuth::new(api_keys);
    
    // Rate limiter: 10 requests per minute
    // 10 tokens capacity, 10/60 = 0.166... tokens/second refill rate
    let rate_limiter = RateLimiter::new(redis_cache, 10.0, 10.0 / 60.0);
    
    let server_address = config.server_address();
    tracing::info!("🔐 Auth Service starting on http://{}", server_address);
    tracing::info!("🛡️  API Key authentication: ENABLED");
    tracing::info!("🛡️  Rate limiting: 10 requests per minute per IP");
    tracing::info!("📍 IP detection: X-Real-IP > X-Forwarded-For > Connection IP");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .wrap(api_key_auth.clone())  // First: Check API key
            .wrap(rate_limiter.clone())  // Then: Rate limit by real IP
            .configure(api::routes::configure)
    })
    .bind(&server_address)?
    .run()
    .await
}
