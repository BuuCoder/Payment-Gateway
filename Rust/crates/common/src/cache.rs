use redis::{Client, Commands, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub fn get_connection(&self) -> Result<redis::Connection, RedisError> {
        self.client.get_connection()
    }

    // Get cached value
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let mut conn = self.get_connection()?;
        let value: Option<String> = conn.get(key)?;
        
        match value {
            Some(v) => {
                let parsed = serde_json::from_str(&v)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON parse error", e.to_string())))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    // Set cached value with TTL
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), RedisError> {
        let mut conn = self.get_connection()?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON serialize error", e.to_string())))?;
        
        conn.set_ex(key, serialized, ttl_seconds)
    }

    // Delete cached value
    pub fn delete(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.get_connection()?;
        conn.del(key)
    }

    // Check if key exists
    pub fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.get_connection()?;
        conn.exists(key)
    }

    // Set with no expiration
    pub fn set_permanent<T: Serialize>(&self, key: &str, value: &T) -> Result<(), RedisError> {
        let mut conn = self.get_connection()?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON serialize error", e.to_string())))?;
        
        conn.set(key, serialized)
    }

    // Increment counter (for rate limiting)
    pub fn increment(&self, key: &str, ttl_seconds: u64) -> Result<i64, RedisError> {
        let mut conn = self.get_connection()?;
        let count: i64 = conn.incr(key, 1)?;
        
        if count == 1 {
            conn.expire(key, ttl_seconds as i64)?;
        }
        
        Ok(count)
    }

    // Get multiple keys
    pub fn mget<T: for<'de> Deserialize<'de>>(&self, keys: &[String]) -> Result<Vec<Option<T>>, RedisError> {
        let mut conn = self.get_connection()?;
        let values: Vec<Option<String>> = conn.get(keys)?;
        
        let mut results = Vec::new();
        for value in values {
            match value {
                Some(v) => {
                    let parsed = serde_json::from_str(&v)
                        .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON parse error", e.to_string())))?;
                    results.push(Some(parsed));
                }
                None => results.push(None),
            }
        }
        
        Ok(results)
    }
}

// Cache key builders
pub fn payment_cache_key(payment_id: &str) -> String {
    format!("payment:{}", payment_id)
}

pub fn user_cache_key(user_id: i64) -> String {
    format!("user:{}", user_id)
}

pub fn rate_limit_key(user_id: i64, action: &str) -> String {
    format!("rate_limit:{}:{}", user_id, action)
}
