use sqlx::{MySqlPool, Result};
use crate::domain::user::User;

#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<User>> {
        sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }
}
