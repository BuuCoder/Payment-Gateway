use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub service_name: String,
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "service".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid number"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
