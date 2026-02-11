use sqlx::{MySqlPool, Pool, MySql};
use anyhow::Result;

pub async fn create_pool(database_url: &str) -> Result<Pool<MySql>> {
    let pool = MySqlPool::connect(database_url).await?;
    Ok(pool)
}
