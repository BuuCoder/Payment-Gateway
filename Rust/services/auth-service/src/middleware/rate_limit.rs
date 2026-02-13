use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use common::cache::RedisCache;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: u64,
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Self::current_timestamp(),
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn refill(&mut self, capacity: f64, refill_rate: f64) {
        let now = Self::current_timestamp();
        let elapsed = (now - self.last_refill) as f64;
        
        let tokens_to_add = elapsed * refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(capacity);
        self.last_refill = now;
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn time_until_available(&self, tokens: f64, refill_rate: f64) -> u64 {
        if self.tokens >= tokens {
            0
        } else {
            let tokens_needed = tokens - self.tokens;
            (tokens_needed / refill_rate).ceil() as u64
        }
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    redis_cache: RedisCache,
    capacity: f64,
    refill_rate: f64,
}

impl RateLimiter {
    /// Create rate limiter for auth endpoints
    /// 
    /// # Arguments
    /// * `redis_cache` - Redis cache instance
    /// * `capacity` - Maximum tokens (burst size)
    /// * `refill_rate` - Tokens per second
    /// 
    /// # Example
    /// ```
    /// // 10 attempts capacity, 1 token/second (60 attempts/minute)
    /// RateLimiter::new(redis_cache, 10.0, 1.0)
    /// ```
    pub fn new(redis_cache: RedisCache, capacity: f64, refill_rate: f64) -> Self {
        Self {
            redis_cache,
            capacity,
            refill_rate,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            redis_cache: self.redis_cache.clone(),
            capacity: self.capacity,
            refill_rate: self.refill_rate,
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    redis_cache: RedisCache,
    capacity: f64,
    refill_rate: f64,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        // Skip rate limiting for health check endpoints
        if path == "/health" || path == "/health/" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }
        
        // Priority 1: X-Real-IP header (from trusted backend with API key)
        // Priority 2: X-Forwarded-For header (first IP in chain)
        // Priority 3: Connection IP
        let ip = req
            .headers()
            .get("X-Real-IP")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                req.headers()
                    .get("X-Forwarded-For")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.split(',').next())
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| {
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string()
            });

        let path = req.path().to_string();
        let key = format!("rate_limit:auth:{}:{}", ip, path);
        
        let redis_cache = self.redis_cache.clone();
        let capacity = self.capacity;
        let refill_rate = self.refill_rate;
        
        let fut = self.service.call(req);

        Box::pin(async move {
            // Get or create token bucket
            let mut bucket: TokenBucket = match redis_cache.get(&key) {
                Ok(Some(b)) => b,
                _ => TokenBucket::new(capacity),
            };

            bucket.refill(capacity, refill_rate);

            if !bucket.try_consume(1.0) {
                let retry_after = bucket.time_until_available(1.0, refill_rate);
                
                tracing::warn!(
                    "Rate limit exceeded for IP {} on {} (tokens: {:.2})",
                    ip, path, bucket.tokens
                );
                
                let response = HttpResponse::TooManyRequests()
                    .insert_header(("X-RateLimit-Limit", capacity.to_string()))
                    .insert_header(("X-RateLimit-Remaining", bucket.tokens.floor().to_string()))
                    .insert_header(("X-RateLimit-Retry-After", retry_after.to_string()))
                    .json(serde_json::json!({
                        "error": "Too many attempts. Please try again later.",
                        "retry_after_seconds": retry_after,
                        "limit": capacity,
                        "remaining": bucket.tokens.floor()
                    }));
                
                let _ = redis_cache.set(&key, &bucket, 180);
                
                return Ok(ServiceResponse::new(
                    fut.await?.request().clone(),
                    response.map_into_right_body()
                ));
            }

            // Save bucket state
            if let Err(e) = redis_cache.set(&key, &bucket, 180) {
                tracing::error!("Failed to save token bucket: {}", e);
            }

            let mut response = fut.await?;
            let headers = response.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-limit"),
                actix_web::http::header::HeaderValue::from_str(&capacity.to_string()).unwrap()
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                actix_web::http::header::HeaderValue::from_str(&bucket.tokens.floor().to_string()).unwrap()
            );

            Ok(response.map_into_left_body())
        })
    }
}
