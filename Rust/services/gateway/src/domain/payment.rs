use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: i32,
    pub user_id: i32,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Succeeded,
    Failed,
    Canceled,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Succeeded => "succeeded",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Canceled => "canceled",
        }
    }
}

impl From<String> for PaymentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "succeeded" => PaymentStatus::Succeeded,
            "failed" => PaymentStatus::Failed,
            "canceled" => PaymentStatus::Canceled,
            _ => PaymentStatus::Pending,
        }
    }
}
