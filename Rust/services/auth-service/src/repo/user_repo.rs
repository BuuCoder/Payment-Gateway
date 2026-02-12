use sqlx::MySqlPool;
use anyhow::Result;
use crate::domain::User;

#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, password FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, password FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create(&self, name: &str, email: &str, hashed_password: &str) -> Result<i32> {
        let result = sqlx::query(
            "INSERT INTO users (name, email, password) VALUES (?, ?, ?)"
        )
        .bind(name)
        .bind(email)
        .bind(hashed_password)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }
}
