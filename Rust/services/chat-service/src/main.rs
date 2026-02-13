mod api;
mod domain;
mod repo;
mod websocket;
mod redis_listener;
mod middleware;

use actix::Actor;
use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use api::{configure_routes, AppState};
use repo::{MessageRepository, RoomRepository};
use websocket::ChatServer;
use redis_listener::RedisListener;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Chat Service...");

    // Load configuration
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root@localhost:3306/rustdb".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let server_port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8084".to_string())
        .parse::<u16>()?;

    // Initialize database pool
    let db_pool = db::pool::create_pool(&database_url).await?;
    info!("Database pool created");

    // Initialize Redis connection
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis_conn = redis_client.get_multiplexed_tokio_connection().await?;
    let redis_conn_listener = redis_client.get_multiplexed_tokio_connection().await?;
    info!("Redis connection established");

    // Initialize repositories
    let message_repo = MessageRepository::new(db_pool.clone());
    let room_repo = RoomRepository::new(db_pool.clone());

    // Start chat server actor
    let chat_server = ChatServer::new(redis_conn).start();
    info!("Chat server actor started");

    // Start Redis listener for cross-instance synchronization
    let redis_listener = RedisListener::new(redis_conn_listener, chat_server.clone());
    tokio::spawn(async move {
        redis_listener.start().await;
    });
    info!("Redis listener started");

    // Create app state
    let app_state = web::Data::new(AppState {
        chat_server,
        message_repo,
        room_repo,
    });

    info!("Starting HTTP server on port {}", server_port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .configure(configure_routes)
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await?;

    Ok(())
}
