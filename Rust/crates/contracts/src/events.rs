use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreatedEvent {
    pub user_id: i32,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserUpdatedEvent {
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
}
