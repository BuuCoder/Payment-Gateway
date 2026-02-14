use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Message,
    Typing,
    RoomAction,
}

impl EventType {
    pub fn capacity(&self) -> f64 {
        match self {
            EventType::Message => 10.0,      // 10 messages burst
            EventType::Typing => 5.0,        // 5 typing events burst
            EventType::RoomAction => 20.0,   // 20 room actions burst
        }
    }

    pub fn refill_rate(&self) -> f64 {
        match self {
            EventType::Message => 1.0,       // 1 token/second = 10 messages/10s
            EventType::Typing => 0.5,        // 0.5 token/second = 5 events/10s
            EventType::RoomAction => 0.33,   // 0.33 token/second = 20 events/60s
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EventType::Message => "message",
            EventType::Typing => "typing",
            EventType::RoomAction => "room_action",
        }
    }
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        // Add tokens based on elapsed time
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, amount: f64) -> Result<(), f64> {
        self.refill();
        
        if self.tokens >= amount {
            self.tokens -= amount;
            Ok(())
        } else {
            // Calculate how long until we have enough tokens
            let needed = amount - self.tokens;
            let wait_time = needed / self.refill_rate;
            Err(wait_time)
        }
    }
}

pub struct RateLimiter {
    // user_id -> (event_type -> bucket)
    buckets: HashMap<i64, HashMap<EventType, TokenBucket>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    pub fn check_rate_limit(&mut self, user_id: i64, event_type: EventType) -> Result<(), f64> {
        let user_buckets = self.buckets.entry(user_id).or_insert_with(HashMap::new);
        
        let bucket = user_buckets.entry(event_type).or_insert_with(|| {
            TokenBucket::new(event_type.capacity(), event_type.refill_rate())
        });

        bucket.try_consume(1.0)
    }

    pub fn cleanup_old_users(&mut self, active_users: &[i64]) {
        // Remove buckets for users who are no longer connected
        let active_set: std::collections::HashSet<_> = active_users.iter().copied().collect();
        self.buckets.retain(|user_id, _| active_set.contains(user_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_burst() {
        let mut bucket = TokenBucket::new(10.0, 1.0);
        
        // Should allow 10 rapid requests (burst)
        for _ in 0..10 {
            assert!(bucket.try_consume(1.0).is_ok());
        }
        
        // 11th request should fail
        assert!(bucket.try_consume(1.0).is_err());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 1.0);
        
        // Consume all tokens
        for _ in 0..10 {
            bucket.try_consume(1.0).unwrap();
        }
        
        // Wait for refill (1 token/second)
        thread::sleep(Duration::from_millis(2100));
        
        // Should have ~2 tokens now
        assert!(bucket.try_consume(1.0).is_ok());
        assert!(bucket.try_consume(1.0).is_ok());
        assert!(bucket.try_consume(1.0).is_err());
    }

    #[test]
    fn test_rate_limiter_per_user() {
        let mut limiter = RateLimiter::new();
        
        // User 1 can send 10 messages
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(1, EventType::Message).is_ok());
        }
        assert!(limiter.check_rate_limit(1, EventType::Message).is_err());
        
        // User 2 still has full quota
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(2, EventType::Message).is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_per_event_type() {
        let mut limiter = RateLimiter::new();
        
        // Consume all message tokens
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(1, EventType::Message).is_ok());
        }
        assert!(limiter.check_rate_limit(1, EventType::Message).is_err());
        
        // Typing tokens should still be available
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(1, EventType::Typing).is_ok());
        }
    }
}
