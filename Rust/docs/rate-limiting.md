# Rate Limiting (Token Bucket)

## 🔍 Rate Limiting là gì?

**Rate Limiting** là kỹ thuật giới hạn số lượng requests mà một user/IP có thể gửi trong một khoảng thời gian để bảo vệ hệ thống khỏi abuse và overload.

**Ví dụ đơn giản:**
- Giống như ATM giới hạn 3 lần nhập sai PIN: bảo vệ tài khoản khỏi brute force
- Token Bucket = túi chứa token (vé), mỗi request tốn 1 token
- Hết token → Phải chờ refill → Ngăn spam

---

## 🎯 Phục vụ vấn đề gì?

### Vấn đề: Không có Rate Limiting

```
❌ Không có Rate Limiting:
Attacker → 10,000 login attempts/second → Server
           ↓
           Server quá tải → Crash
           User thật không truy cập được
           Brute force thành công

Vấn đề:
- Brute force attacks: thử hàng nghìn passwords
- DDoS attacks: làm quá tải server
- API abuse: user spam requests
- Resource exhaustion: database/CPU quá tải
```

### Giải pháp: Token Bucket Rate Limiting

```
✅ Có Rate Limiting:
User → Request → Rate Limiter → Server
                 ↓
                 Check: Còn token?
                 ├─ Có → Cho qua (trừ 1 token)
                 └─ Không → 429 Too Many Requests

Lợi ích:
- Brute force: Chỉ 10 attempts/minute → Cần 100 phút để thử 1000 passwords
- DDoS: Giới hạn requests per IP → Không quá tải
- Fair usage: Mỗi user có quota riêng
- Resource protection: Server không bị overwhelm
```

---

## 🏗️ Token Bucket Algorithm

### Cách hoạt động

```
┌─────────────────────────────────────┐
│         Token Bucket                │
│                                     │
│  Capacity: 10 tokens                │
│  ┌───┬───┬───┬───┬───┐             │
│  │ ● │ ● │ ● │ ● │ ● │             │
│  ├───┼───┼───┼───┼───┤             │
│  │ ● │ ● │ ● │ ● │ ● │             │
│  └───┴───┴───┴───┴───┘             │
│                                     │
│  Refill: 0.166 tokens/second        │
│         (10 tokens/60 seconds)      │
└─────────────────────────────────────┘

Request 1 → Trừ 1 token → 9 tokens còn lại
Request 2 → Trừ 1 token → 8 tokens còn lại
...
Request 10 → Trừ 1 token → 0 tokens còn lại
Request 11 → Không có token → 429 Too Many Requests

Sau 6 giây → Refill 1 token → Request 12 OK
Sau 60 giây → Refill đầy 10 tokens → 10 requests OK
```

### Tham số

| Tham số | Giá trị | Ý nghĩa |
|---------|---------|---------|
| **Capacity** | 10 tokens | Số requests burst tối đa (ngay lập tức) |
| **Refill Rate** | 0.166 tokens/s | Tốc độ refill (10 tokens/60s = 10 req/min) |
| **Cost** | 1 token/request | Mỗi request tốn 1 token |

**Kết quả:**
- ✅ Cho phép **10 requests burst** (ngay lập tức)
- ✅ Sau đó **10 requests/minute** sustained
- ✅ Sau 60 giây, bucket đầy lại (10 tokens)

---

## 🏗️ Vai trò trong Source Code

### 1. **Gateway Rate Limiting (Port 8080)**
```rust
// Giới hạn per user (JWT user_id)
RateLimiter::new(
    redis_cache,
    10.0,        // Capacity: 10 tokens
    10.0 / 60.0  // Refill: 10 tokens/60 seconds
)

// Identifier: JWT user_id
let key = format!("ratelimit:user:{}", claims.user_id);
```

### 2. **Auth Service Rate Limiting (Port 8081)**
```rust
// Giới hạn per IP (brute force protection)
RateLimiter::new(
    redis_cache,
    10.0,        // Capacity: 10 tokens
    10.0 / 60.0  // Refill: 10 tokens/60 seconds
)

// Identifier: IP address (X-Real-IP > X-Forwarded-For > Connection IP)
let key = format!("ratelimit:ip:{}", client_ip);
```

### 3. **Endpoints được bảo vệ**
- **Auth Service**: `/login`, `/register`
- **Gateway**: Tất cả `/api/*` endpoints

---

## 📖 Cách sử dụng

### Configuration

```rust
// Strict rate limiting (10 requests/minute)
let rate_limiter = RateLimiter::new(
    redis_cache,
    10.0,        // Capacity
    10.0 / 60.0  // Refill rate
);

// Generous rate limiting (60 requests/minute)
let rate_limiter = RateLimiter::new(
    redis_cache,
    10.0,   // Capacity (burst)
    1.0     // Refill rate (60 tokens/60s)
);

// Internal APIs (600 requests/minute)
let rate_limiter = RateLimiter::new(
    redis_cache,
    100.0,  // Capacity (burst)
    10.0    // Refill rate (600 tokens/60s)
);
```

---

## 🧪 Testing

### Test Auth Service Rate Limiting

```bash
# Test login rate limit (10 requests per minute)
for i in {1..10}; do
  curl -X POST http://localhost:8081/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
    -H "X-Real-IP: 192.168.1.100" \
    -d '{"email":"test@example.com","password":"wrong"}'
  echo "Request $i: OK"
done

# 11th attempt → 429 Too Many Requests
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"test@example.com","password":"wrong"}'

# Response:
# {
#   "error": "Too many attempts. Please try again later.",
#   "retry_after_seconds": 5,
#   "limit": 10,
#   "remaining": 0
# }

# Wait 60 seconds (10 tokens refilled)
sleep 60

# Should work again
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: iouWN3RyYqinQVLodUloltG2aEzzCIE" \
  -H "X-Real-IP: 192.168.1.100" \
  -d '{"email":"user@example.com","password":"password"}'
```

### Test Gateway Rate Limiting

```bash
# Login lấy token
TOKEN=$(curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "X-API-Key: dev-key-12345" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.token')

# Test burst (10 requests OK)
for i in {1..10}; do
  curl -H "Authorization: Bearer $TOKEN" \
    http://localhost:8080/api/v1/payments
  echo "Request $i: OK"
done

# Request 11 → 429 Too Many Requests
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/payments

# Wait 60 seconds (10 tokens refilled)
sleep 60

# Should work again (10 more requests)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/payments
```

---

## 📊 Services có Rate Limiting

| Service | Port | Capacity | Refill Rate | Identifier | Endpoints |
|---------|------|----------|-------------|------------|-----------|
| **Gateway** | 8080 | 10 tokens | 10 req/min | JWT user_id | Tất cả `/api/*` |
| **Auth Service** | 8081 | 10 tokens | 10 req/min | IP address | `/login`, `/register` |

---

## 🛡️ Security Benefits

### 1. Brute Force Protection

**Scenario:** Attacker thử brute force login

```
Không có Rate Limiting:
- Attacker thử 10,000 passwords/second
- Tìm được password trong vài phút

Có Rate Limiting (10 req/min):
- Attacker chỉ thử được 10 passwords/minute
- Cần ~100 phút để thử 1,000 passwords
- Kết hợp account lockout → Bảo vệ tốt hơn
```

### 2. API Quota Management

**Scenario:** User spam requests

```
Gateway Rate Limiting:
- User A: 10 requests/minute
- User B: 10 requests/minute
- User C: 10 requests/minute

Auth Service Rate Limiting:
- IP 1.1.1.1: 10 requests/minute
- IP 2.2.2.2: 10 requests/minute

→ Fair usage policy enforcement
→ Ngăn chặn abuse và excessive usage
```

### 3. DDoS Mitigation

**Scenario:** DDoS attack từ nhiều IPs

```
Không có Rate Limiting:
- 1000 IPs × 100 req/s = 100,000 req/s
- Server overwhelmed → Crash

Có Rate Limiting:
- 1000 IPs × 10 req/min = ~167 req/s
- Server xử lý được → Stable
- Attacker không đạt mục đích
```

---

## ✅ Khi nào nên dùng?

| Tình huống | Nên dùng? | Lý do |
|------------|-----------|-------|
| Auth endpoints (login/register) | ✅ Có | Prevent brute force attacks |
| Public APIs cần bảo vệ | ✅ Có | Ngăn abuse, fair usage |
| Endpoints tốn tài nguyên (write, search) | ✅ Có | Bảo vệ database/CPU |
| Cần kiểm soát per-user/IP usage | ✅ Có | Quota management |
| APIs với strict quota | ✅ Có | 10 req/min, 100 req/hour, etc. |
| Internal services đã tin cậy | ❌ Không | Overhead không cần thiết |
| Endpoints đơn giản (health check) | ❌ Không | Không cần giới hạn |

---

## 🔧 Advanced Configuration

### Different Limits per Endpoint

```rust
// Login: Strict (5 req/min)
let login_limiter = RateLimiter::new(redis, 5.0, 5.0 / 60.0);

// Register: Moderate (10 req/min)
let register_limiter = RateLimiter::new(redis, 10.0, 10.0 / 60.0);

// API calls: Generous (60 req/min)
let api_limiter = RateLimiter::new(redis, 10.0, 1.0);
```

### Different Limits per User Tier

```rust
// Free tier: 10 req/min
let free_limiter = RateLimiter::new(redis, 10.0, 10.0 / 60.0);

// Pro tier: 100 req/min
let pro_limiter = RateLimiter::new(redis, 20.0, 100.0 / 60.0);

// Enterprise: 1000 req/min
let enterprise_limiter = RateLimiter::new(redis, 100.0, 1000.0 / 60.0);
```

---

## 🚨 Error Handling

### Rate Limit Response

```json
{
  "error": "Too many attempts. Please try again later.",
  "retry_after_seconds": 5,
  "limit": 10,
  "remaining": 0
}
```

### Response Headers

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1709472000
Retry-After: 5
```

### Client Handling

```javascript
// Client-side retry logic
async function callAPI() {
  try {
    const response = await fetch('/api/payments', {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (response.status === 429) {
      const retryAfter = response.headers.get('Retry-After');
      console.log(`Rate limited. Retry after ${retryAfter}s`);
      
      // Wait and retry
      await sleep(retryAfter * 1000);
      return callAPI();
    }
    
    return response.json();
  } catch (error) {
    console.error('API call failed:', error);
  }
}
```

---

## 📊 Monitoring

### Redis Keys

```bash
# Xem tất cả rate limit keys
redis-cli --scan --pattern "ratelimit:*"

# Output:
# ratelimit:user:123
# ratelimit:user:456
# ratelimit:ip:192.168.1.100

# Xem tokens còn lại
redis-cli GET "ratelimit:user:123"
# Output: 7 (còn 7 tokens)

# Xem TTL
redis-cli TTL "ratelimit:user:123"
# Output: 45 (reset sau 45 giây)
```

### Metrics

```rust
// Log rate limit events
if rate_limited {
    warn!(
        "Rate limit exceeded: user_id={}, ip={}, endpoint={}",
        user_id, ip, endpoint
    );
    
    // Increment metrics
    metrics.rate_limit_exceeded.inc();
}
```

---

## 💡 Tóm tắt

**Rate Limiting với Token Bucket** bảo vệ hệ thống khỏi abuse:
- **Mục đích**: Ngăn brute force, DDoS, API abuse
- **Cách hoạt động**: Token bucket với capacity và refill rate
- **Vai trò**: Bảo vệ Auth endpoints, API quota management, fair usage
- **Kết quả**: Brute force cần 100 phút thay vì vài phút, server stable, fair usage cho tất cả users
