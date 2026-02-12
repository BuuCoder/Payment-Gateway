use sqlx::MySqlPool;
use anyhow::Result;
use crate::domain::Payment;

#[derive(Clone)]
pub struct PaymentRepository {
    pool: MySqlPool,
}

impl PaymentRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i32,
        amount: f64,
        currency: &str,
        status: &str,
        payment_method: &str,
        stripe_payment_intent_id: &str,
        stripe_client_secret: &str,
    ) -> Result<i32> {
        let result = sqlx::query(
            "INSERT INTO payments (user_id, amount, currency, status, payment_method, stripe_payment_intent_id, stripe_client_secret) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(amount)
        .bind(currency)
        .bind(status)
        .bind(payment_method)
        .bind(stripe_payment_intent_id)
        .bind(stripe_client_secret)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn find_by_stripe_intent_id(&self, intent_id: &str) -> Result<Option<Payment>> {
        let payment = sqlx::query_as::<_, Payment>(
            "SELECT id, user_id, amount, currency, status, payment_method, stripe_payment_intent_id, stripe_client_secret 
             FROM payments WHERE stripe_payment_intent_id = ?"
        )
        .bind(intent_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(payment)
    }

    pub async fn update_status(&self, intent_id: &str, status: &str) -> Result<()> {
        sqlx::query(
            "UPDATE payments SET status = ? WHERE stripe_payment_intent_id = ?"
        )
        .bind(status)
        .bind(intent_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
