use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentCreatedEvent {
    pub payment_id: i32,
    pub user_id: i32,
    pub amount: f64,
    pub status: String,
    pub timestamp: String,
}
