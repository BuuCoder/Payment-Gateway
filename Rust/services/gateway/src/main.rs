mod routes;
mod middleware;
mod clients;
mod handlers;
mod domain;
mod repo;
mod service;

use actix_web::{web, App, HttpServer};
use std::env;
use messaging::kafka_producer::KafkaProducer;
use clients::StripeClient;
use repo::PaymentRepository;
use service::PaymentService;
use common::cache::RedisCache;
use middleware::rate_limit::RateLimiter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let stripe_api_key = env::var("STRIPE_API_KEY").expect("STRIPE_API_KEY must be set");
    
    // Create database pool
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");
    
    // Create Redis cache
    let redis_cache = RedisCache::new(&redis_url)
        .expect("Failed to create Redis cache");
    
    // Create Kafka producer
    let producer = KafkaProducer::new(&kafka_brokers)
        .expect("Failed to create Kafka producer");
    
    // Create Stripe client
    let stripe_client = StripeClient::new(stripe_api_key);
    
    // Initialize layers
    let payment_repo = PaymentRepository::new(pool.clone());
    let payment_service = PaymentService::new(payment_repo, stripe_client.clone(), producer.clone(), redis_cache.clone());
    
    // Rate limiter: 10 requests capacity, 10/60 = 0.166... tokens/second
    // This allows 10 requests per minute with small burst tolerance
    let rate_limiter = RateLimiter::new(redis_cache.clone(), 10.0, 10.0 / 60.0);
    
    let server_address = env::var("SERVER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        + ":"
        + &env::var("SERVER_PORT").unwrap_or_else(|_| "8083".to_string());
    
    tracing::info!("🚪 Gateway starting on http://{}", server_address);
    tracing::info!("✅ Redis cache connected");
    tracing::info!("🛡️  Rate limiting: 10 requests per minute per user");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(producer.clone()))
            .app_data(web::Data::new(stripe_client.clone()))
            .app_data(web::Data::new(payment_service.clone()))
            .app_data(web::Data::new(redis_cache.clone()))
            .wrap(rate_limiter.clone())
            .configure(routes::configure)
    })
    .bind(&server_address)?
    .run()
    .await
}
